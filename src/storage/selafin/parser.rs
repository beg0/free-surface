//! # Selafin Parser
//!
//! Parses the Selafin binary file format.
//! Supports both big-endian and little-endian files, and both f32 and f64 meshes.

use super::container::{FloatSize, SlfArray1D, SlfArray2D};
use super::geometry::SlfGeometry;
use super::variable::{SlfVariable, TimeSerie, VariableEvolution};
use super::Selafin;
use binrw::{BinReaderExt, Endian};
use chrono::NaiveDateTime;
use std::cmp::max;
use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom};

/// Indexes for IPARAM header array
enum IParams {
    XOrigin = 2,
    YOrigin = 3,
    PlanesCnt = 6,     // Number of planes on the vertical (3D computation)
    BoundariesCnt = 7, // Number of boundary points (for parallel computations)
    IfaceCnt = 8,      // Number of interface points (for parallel computations)
    HasDateTime = 9,
}

// ---------------------------------------------------------------------------
// Low-level record helpers
// ---------------------------------------------------------------------------

/// Read one Fortran-style record: u32 length, `length` bytes of data, u32 control.
/// Returns the raw bytes if both length fields agree; otherwise returns an error.
fn read_record<R: Read + Seek>(reader: &mut R, endian: Endian) -> binrw::BinResult<Vec<u8>> {
    let read_u32 = |r: &mut R| -> binrw::BinResult<u32> {
        match endian {
            Endian::Big => r.read_be::<u32>(),
            Endian::Little => r.read_le::<u32>(),
        }
    };

    let len = read_u32(reader)?;
    let mut data = vec![0u8; len as usize];
    reader.read_exact(&mut data)?;
    let ctrl = read_u32(reader)?;

    if len != ctrl {
        return Err(binrw::Error::AssertFail {
            pos: reader.stream_position()?,
            message: format!("Record length mismatch: header={len} trailer={ctrl}"),
        });
    }
    Ok(data)
}

/// Peek at the very first record to decide endianness.
/// Returns `Ok(endian)` with the endianness that yields a consistent record.
fn detect_endianness<R: Read + Seek>(reader: &mut R) -> binrw::BinResult<Endian> {
    let start = reader.stream_position()?;

    // Try native endian first (cfg-determined), then the opposite.
    #[cfg(target_endian = "little")]
    let candidates = [Endian::Little, Endian::Big];
    #[cfg(target_endian = "big")]
    let candidates = [Endian::Big, Endian::Little];

    for &endian in &candidates {
        reader.seek(SeekFrom::Start(start))?;
        if read_record(reader, endian).is_ok() {
            reader.seek(SeekFrom::Start(start))?;
            return Ok(endian);
        }
    }

    reader.seek(SeekFrom::Start(start))?;
    Err(binrw::Error::AssertFail {
        pos: start,
        message: "Cannot determine file endianness from the first record".into(),
    })
}

// ---------------------------------------------------------------------------
// Typed reads from a byte slice
// ---------------------------------------------------------------------------

fn read_u32s(data: &[u8], endian: Endian) -> Vec<u32> {
    data.chunks_exact(4)
        .map(|b| {
            let arr = [b[0], b[1], b[2], b[3]];
            match endian {
                Endian::Big => u32::from_be_bytes(arr),
                Endian::Little => u32::from_le_bytes(arr),
            }
        })
        .collect()
}

#[allow(dead_code)]
fn read_i32s(data: &[u8], endian: Endian) -> Vec<i32> {
    data.chunks_exact(4)
        .map(|b| {
            let arr = [b[0], b[1], b[2], b[3]];
            match endian {
                Endian::Big => i32::from_be_bytes(arr),
                Endian::Little => i32::from_le_bytes(arr),
            }
        })
        .collect()
}

fn read_f32s(data: &[u8], endian: Endian) -> Vec<f32> {
    data.chunks_exact(4)
        .map(|b| {
            let arr = [b[0], b[1], b[2], b[3]];
            match endian {
                Endian::Big => f32::from_be_bytes(arr),
                Endian::Little => f32::from_le_bytes(arr),
            }
        })
        .collect()
}

fn read_f64s(data: &[u8], endian: Endian) -> Vec<f64> {
    data.chunks_exact(8)
        .map(|b| {
            let arr = [b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]];
            match endian {
                Endian::Big => f64::from_be_bytes(arr),
                Endian::Little => f64::from_le_bytes(arr),
            }
        })
        .collect()
}

trait MultipleReader: Sized {
    fn vec_from_bytes_endian(data: &[u8], endian: Endian) -> Vec<Self>;
}

impl MultipleReader for u32 {
    fn vec_from_bytes_endian(data: &[u8], endian: Endian) -> Vec<Self> {
        read_u32s(data, endian)
    }
}

impl MultipleReader for f32 {
    fn vec_from_bytes_endian(data: &[u8], endian: Endian) -> Vec<Self> {
        read_f32s(data, endian)
    }
}
impl MultipleReader for f64 {
    fn vec_from_bytes_endian(data: &[u8], endian: Endian) -> Vec<Self> {
        read_f64s(data, endian)
    }
}

// ---------------------------------------------------------------------------
// String helpers
// ---------------------------------------------------------------------------

fn ascii_record_to_string(data: &[u8]) -> String {
    String::from_utf8_lossy(data).trim_end().to_string()
}

/// Split a fixed-width byte slice into chunks of `width`, trim each, return strings.
fn split_fixed_strings(data: &[u8], width: usize) -> Vec<String> {
    data.chunks(width)
        .map(|c| String::from_utf8_lossy(c).trim_end().to_string())
        .collect()
}

// ---------------------------------------------------------------------------
// Float-size detection
// ---------------------------------------------------------------------------

/// After all integer geometry records have been consumed, the next record
/// contains `npoin3` floats.  We know npoin3, so we can check which size fits.
fn detect_float_size<R: Read + Seek>(
    reader: &mut R,
    endian: Endian,
    npoin3: usize,
) -> binrw::BinResult<FloatSize> {
    let start = reader.stream_position()?;

    let data = read_record(reader, endian)?;
    reader.seek(SeekFrom::Start(start))?;

    let expected_f32 = npoin3 * size_of::<f32>();
    let expected_f64 = npoin3 * size_of::<f64>();

    if data.len() == expected_f32 {
        Ok(FloatSize::F32)
    } else if data.len() == expected_f64 {
        Ok(FloatSize::F64)
    } else {
        Err(binrw::Error::AssertFail {
            pos: start,
            message: format!(
                "Cannot determine float size: record is {} bytes, \
                 expected {} (f32) or {} (f64) for npoin3={}",
                data.len(),
                expected_f32,
                expected_f64,
                npoin3
            ),
        })
    }
}

// ---------------------------------------------------------------------------
// Datetime parsing
// ---------------------------------------------------------------------------

/// Convert 6 × u32 `[YYYY, MM, DD, HH, MM, SS]` into a validated `NaiveDateTime`.
fn parse_datetime(vals: &[u32], pos: u64) -> binrw::BinResult<NaiveDateTime> {
    let (year, month, day, hour, min, sec) =
        (vals[0] as i32, vals[1], vals[2], vals[3], vals[4], vals[5]);

    let date = chrono::NaiveDate::from_ymd_opt(year, month, day).ok_or_else(|| {
        binrw::Error::AssertFail {
            pos,
            message: format!("Invalid date in datetime record: {year:04}-{month:02}-{day:02}"),
        }
    })?;

    let time = chrono::NaiveTime::from_hms_opt(hour, min, sec).ok_or_else(|| {
        binrw::Error::AssertFail {
            pos,
            message: format!("Invalid time in datetime record: {hour:02}:{min:02}:{sec:02}"),
        }
    })?;

    Ok(NaiveDateTime::new(date, time))
}

// ---------------------------------------------------------------------------
// Main parser entry point
// ---------------------------------------------------------------------------

/// Parse a Selafin binary file from any `Read + Seek` source.
///
/// # Example
/// ```no_run
/// use std::fs::File;
/// use std::io::BufReader;
/// use free_surface::storage::selafin;
///
/// let f = File::open("my_file.slf").unwrap();
/// let selafin = selafin::parse(BufReader::new(f)).unwrap();
/// ```
pub fn parse<R: Read + Seek>(mut reader: R) -> binrw::BinResult<Selafin> {
    // -----------------------------------------------------------------------
    // 1. Detect endianness
    // -----------------------------------------------------------------------
    let endian = detect_endianness(&mut reader)?;

    // -----------------------------------------------------------------------
    // 2. Metadata
    // -----------------------------------------------------------------------

    // 1.1 - Title (80-character string)
    let title = {
        let data = read_record(&mut reader, endian)?;
        if data.len() < 80 {
            return Err(binrw::Error::AssertFail {
                pos: reader.stream_position()?,
                message: format!("Title record too short: {} bytes", data.len()),
            });
        }
        ascii_record_to_string(&data[..80])
    };

    // 1.2 - (nvar, ncld) tuple
    let (nvar, ncld) = {
        let data = read_record(&mut reader, endian)?;
        let vals = read_u32s(&data, endian);
        if vals.len() < 2 {
            return Err(binrw::Error::AssertFail {
                pos: reader.stream_position()?,
                message: "Variable count record too short".into(),
            });
        }
        (vals[0] as usize, vals[1] as usize)
    };

    // 1.3 - `var` entries (nvar records, each 32 bytes: 16-char name + 16-char unit)
    let var = read_variable_records(&mut reader, endian, nvar)?;

    // 1.4 - `cld` entries
    let cld = read_variable_records(&mut reader, endian, ncld)?;

    // 1.5 - iparam (10 × i32)
    let iparams = {
        let data = read_record(&mut reader, endian)?;
        let vals = read_u32s(&data, endian);
        if vals.len() != 10 {
            return Err(binrw::Error::AssertFail {
                pos: reader.stream_position()?,
                message: format!("iparam record has {} values, expected 10", vals.len()),
            });
        }
        vals
    };

    // 1.5.1 - origin point
    let origin = (
        iparams[IParams::XOrigin as usize],
        iparams[IParams::YOrigin as usize],
    );

    // 1.6 - Optional datetime record when iparam[9] == 1
    let datetime = if iparams[IParams::HasDateTime as usize] == 1 {
        let data = read_record(&mut reader, endian)?;
        let vals = read_u32s(&data, endian);
        if vals.len() < 6 {
            return Err(binrw::Error::AssertFail {
                pos: reader.stream_position()?,
                message: format!("datetime record has only {} values, expected 6", vals.len()),
            });
        }

        Some(parse_datetime(&vals[..6], reader.stream_position()?)?)
    } else {
        None
    };

    // -----------------------------------------------------------------------
    // 3. Geometry - integer elements
    // -----------------------------------------------------------------------

    // 2.1 - (nelem3, npoin3, npd3, nplan)
    let (nelem3, npoin3, npd3, nplan) = {
        let data = read_record(&mut reader, endian)?;
        let vals = read_u32s(&data, endian);
        if vals.len() < 4 {
            return Err(binrw::Error::AssertFail {
                pos: reader.stream_position()?,
                message: format!("Geometry integer header has only {} values", vals.len()),
            });
        }

        let nplan = max(vals[3], 1);

        if nplan != max(iparams[IParams::PlanesCnt as usize], 1) {
            return Err(binrw::Error::AssertFail {
                pos: reader.stream_position()?,
                message: format!(
                    "Inconsistent number of planes, IPARAMS says {}, Geometry says {}",
                    iparams[IParams::PlanesCnt as usize],
                    nplan
                ),
            });
        }

        (vals[0] as usize, vals[1] as usize, vals[2] as usize, nplan)
    };

    // 2.2 - ikle3 (nelem3 × npd3 connectivity table)
    let mesh = {
        let expected = nelem3 * npd3;
        let data = read_record(&mut reader, endian)?;
        let vals = read_u32s(&data, endian);
        if vals.len() != expected {
            return Err(binrw::Error::AssertFail {
                pos: reader.stream_position()?,
                message: format!(
                    "ikle3 record has {} values, expected {}",
                    vals.len(),
                    expected
                ),
            });
        }
        vals.iter().map(|v| v - 1).collect()
    };

    // 2.3 - ipob3 (npoin3 boundary codes)
    // TODO: if iparams[7] or iparams[8] != 0, then it's KNOLG and not IPOBO
    let ipob3 = {
        let expected = npoin3;
        let data = read_record(&mut reader, endian)?;
        let vals = read_u32s(&data, endian);
        if vals.len() != expected {
            return Err(binrw::Error::AssertFail {
                pos: reader.stream_position()?,
                message: format!(
                    "ipob3 record has {} values, expected {}",
                    vals.len(),
                    expected
                ),
            });
        }

        // Values in Slf files are 1-based indexes where '0' means invalid value
        // (e.g. inner node in that case)
        // Let's remove invalid value and make the indexes 0-based
        vals.iter()
            .filter_map(|v| if *v == 0 { None } else { Some(*v - 1) })
            .collect()
    };

    // -----------------------------------------------------------------------
    // 4. Geometry - float mesh (detect f32 vs f64 here)
    // -----------------------------------------------------------------------
    let float_size = detect_float_size(&mut reader, endian, npoin3)?;

    let x_coords_data = read_record(&mut reader, endian)?;
    let y_coords_data = read_record(&mut reader, endian)?;

    let points = match float_size {
        FloatSize::F32 => SlfArray2D::Float {
            x: read_f32s(&x_coords_data, endian),
            y: read_f32s(&y_coords_data, endian),
        },
        FloatSize::F64 => SlfArray2D::Double {
            x: read_f64s(&x_coords_data, endian),
            y: read_f64s(&y_coords_data, endian),
        },
    };

    let (x_len, y_len) = match &points {
        SlfArray2D::Float { x, y } => (x.len(), y.len()),
        SlfArray2D::Double { x, y } => (x.len(), y.len()),
    };

    // Make sure x & y has the same length! Later (in slf.points_count())
    // we are only relying on the size of 'x' vector
    if (x_len != npoin3) || (y_len != npoin3) {
        return Err(binrw::Error::AssertFail {
            pos: reader.stream_position()?,
            message: format!(
                "Inconsistent number of points.
                Header says {} points, x record says {} points, y records says {}",
                npoin3, x_len, y_len,
            ),
        });
    }

    let geo = SlfGeometry::new(points, ipob3, mesh, npd3, nplan).with_parallel_info(
        iparams[IParams::BoundariesCnt as usize],
        iparams[IParams::IfaceCnt as usize],
    );

    // -----------------------------------------------------------------------
    // 5. Optional time history (remaining records, each npoin3 floats)
    //
    // The caller may retrieve these separately; we skip them here and leave
    // the reader positioned at the first time-step record (if any).
    // Uncomment the block below to collect all time-step data eagerly.
    // -----------------------------------------------------------------------
    //

    let results = match float_size {
        FloatSize::F32 => read_history::<f32, R>(&mut reader, &var, &cld, npoin3, endian),
        FloatSize::F64 => read_history::<f64, R>(&mut reader, &var, &cld, npoin3, endian),
    }?;

    Ok(Selafin {
        title,
        origin,
        geo,
        var,
        cld,
        results,
        datetime,
    })
}

/// Return the number of bytes until end of file
///
fn remaining_file_size<R: Seek + ?Sized>(reader: &mut R) -> std::io::Result<u64> {
    let old_pos = reader.stream_position()?;
    let len = reader.seek(SeekFrom::End(0))?;

    // Avoid seeking a third time when we were already at the end of the
    // stream. The branch is usually way cheaper than a seek operation.
    if old_pos != len {
        reader.seek(SeekFrom::Start(old_pos))?;
    }

    Ok(len - old_pos)
}

// ---------------------------------------------------------------------------
// Helper: read N variable-name/unit record pairs
// ---------------------------------------------------------------------------

fn read_variable_records<R: Read + Seek>(
    reader: &mut R,
    endian: Endian,
    count: usize,
) -> binrw::BinResult<Vec<SlfVariable>> {
    let mut vars = Vec::with_capacity(count);
    for _ in 0..count {
        let data = read_record(reader, endian)?;
        // Each record is 32 bytes: 16-char name + 16-char unit
        if data.len() < 32 {
            return Err(binrw::Error::AssertFail {
                pos: reader.stream_position()?,
                message: format!(
                    "Variable record too short: {} bytes (expected ≥ 32)",
                    data.len()
                ),
            });
        }
        let parts = split_fixed_strings(&data, 16);
        #[allow(clippy::get_first)]
        let name = parts.get(0).cloned().unwrap_or_default();
        let unit = parts.get(1).cloned().unwrap_or_default();
        vars.push(SlfVariable::new(&name, &unit));
    }
    Ok(vars)
}

fn read_history<T: MultipleReader + Default + Clone, R: Read + Seek>(
    reader: &mut R,
    var_defs: &Vec<SlfVariable>,
    cld_defs: &Vec<SlfVariable>,
    npoin3: usize,
    endian: Endian,
) -> binrw::BinResult<TimeSerie>
where
    SlfArray1D: From<Vec<T>>,
{
    type PointValues<T> = Vec<T>; // Will become the SlfArray1D when transformed with .into()

    let nb_var = var_defs.len() + cld_defs.len();

    if nb_var == 0 {
        return Ok(TimeSerie::default());
    }

    let mut all_var_defs: Vec<&SlfVariable> = Vec::with_capacity(nb_var);

    for var_def in var_defs {
        all_var_defs.push(var_def)
    }

    for cld_def in cld_defs {
        all_var_defs.push(cld_def)
    }

    let rem_size = remaining_file_size(reader)? as usize;
    let size_time_record = 4 + size_of::<T>() + 4;
    let size_vars_records = 4 + npoin3 * size_of::<T>() + 4;
    let size_per_time_step = size_time_record + nb_var * size_vars_records;

    let history_size = rem_size / size_per_time_step;

    if history_size == 0 {
        return Ok(TimeSerie::default());
    }

    let mut time: Vec<T> = Vec::with_capacity(history_size);

    let mut values_all_var: Vec<Vec<PointValues<T>>> =
        vec![Vec::<PointValues::<T>>::with_capacity(history_size); nb_var];

    for time_idx in 0..history_size {
        let time_data = read_record(reader, endian)?;
        let t = T::vec_from_bytes_endian(&time_data, endian);
        if t.is_empty() {
            return Err(binrw::Error::AssertFail {
                pos: reader.stream_position()?,
                message: format!("Time entry for history #{} is empty", time_idx),
            });
        }

        let mut all_values_for_current_time: Vec<PointValues<T>> = Vec::with_capacity(nb_var);

        for var_def in &all_var_defs {
            let var_data = read_record(reader, endian)?;
            let var_values = T::vec_from_bytes_endian(&var_data, endian);

            if var_values.len() != npoin3 {
                return Err(binrw::Error::AssertFail {
                    pos: reader.stream_position()?,
                    message: format!(
                        "values of Variable '{}' value records for history entry #{} has {} values, expected {}",
                        var_def.name,
                        time_idx,
                        var_values.len(),
                        npoin3
                    ),
                });
            }

            all_values_for_current_time.push(var_values);
        }

        time.push(t.first().cloned().unwrap_or_default());

        for (dst, src) in std::iter::zip(&mut values_all_var, all_values_for_current_time) {
            dst.push(src);
        }
    }

    let mut vars: HashMap<String, VariableEvolution> = HashMap::with_capacity(nb_var);

    for (var_def, values) in std::iter::zip(all_var_defs, values_all_var) {
        let name = var_def.name.clone();
        let ve = VariableEvolution {
            var: var_def.clone(),
            values: values.into_iter().map(|v| v.into()).collect(),
        };

        vars.insert(name, ve);
    }

    Ok(TimeSerie::new(time.into(), vars))
}
// ---------------------------------------------------------------------------
// Re-export a convenience function that opens a file by path
// ---------------------------------------------------------------------------

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

/// Open and parse a Selafin file at `path`.
pub fn parse_file<P: AsRef<Path>>(path: P) -> binrw::BinResult<Selafin> {
    let file = File::open(path).map_err(binrw::Error::Io)?;
    parse(BufReader::new(file))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    // -----------------------------------------------------------------------
    // Helpers to build raw binary buffers
    // -----------------------------------------------------------------------

    /// Wrap `payload` in a Fortran-style record (u32 length + data + u32 length)
    /// using the given endianness.
    fn make_record(payload: &[u8], endian: Endian) -> Vec<u8> {
        let len = payload.len() as u32;
        let len_bytes = match endian {
            Endian::Big => len.to_be_bytes(),
            Endian::Little => len.to_le_bytes(),
        };
        let mut buf = Vec::new();
        buf.extend_from_slice(&len_bytes);
        buf.extend_from_slice(payload);
        buf.extend_from_slice(&len_bytes);
        buf
    }

    fn f32_bytes(v: f32, endian: Endian) -> [u8; 4] {
        match endian {
            Endian::Big => v.to_be_bytes(),
            Endian::Little => v.to_le_bytes(),
        }
    }

    fn f64_bytes(v: f64, endian: Endian) -> [u8; 8] {
        match endian {
            Endian::Big => v.to_be_bytes(),
            Endian::Little => v.to_le_bytes(),
        }
    }

    // -----------------------------------------------------------------------
    // ascii_record_to_string
    // -----------------------------------------------------------------------

    #[test]
    fn ascii_record_trims_trailing_spaces() {
        let s = ascii_record_to_string(b"HELLO WORLD         ");
        assert_eq!(s, "HELLO WORLD");
    }

    #[test]
    fn ascii_record_empty_input_gives_empty_string() {
        assert_eq!(ascii_record_to_string(b""), "");
    }

    #[test]
    fn ascii_record_all_spaces_gives_empty_string() {
        assert_eq!(ascii_record_to_string(b"        "), "");
    }

    #[test]
    fn ascii_record_no_trailing_spaces_unchanged() {
        assert_eq!(ascii_record_to_string(b"EXACT"), "EXACT");
    }

    #[test]
    fn ascii_record_preserves_inner_spaces() {
        assert_eq!(ascii_record_to_string(b"A B C   "), "A B C");
    }

    // -----------------------------------------------------------------------
    // split_fixed_strings
    // -----------------------------------------------------------------------

    #[test]
    fn split_fixed_strings_splits_into_correct_chunks() {
        let data = b"VELOCITY U      M/S             ";
        let parts = split_fixed_strings(data, 16);
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0], "VELOCITY U");
        assert_eq!(parts[1], "M/S");
    }

    #[test]
    fn split_fixed_strings_trims_trailing_spaces_per_chunk() {
        let data = b"DEPTH           M               ";
        let parts = split_fixed_strings(data, 16);
        assert_eq!(parts[0], "DEPTH");
        assert_eq!(parts[1], "M");
    }

    #[test]
    fn split_fixed_strings_handles_exact_width_input() {
        let data = b"ABCDEFGHIJKLMNOP"; // exactly 16 bytes cspell: disable-line
        let parts = split_fixed_strings(data, 16);
        assert_eq!(parts.len(), 1);
        assert_eq!(parts[0], "ABCDEFGHIJKLMNOP"); // cspell: disable-line
    }

    #[test]
    fn split_fixed_strings_empty_input_gives_empty_vec() {
        let parts = split_fixed_strings(b"", 16);
        assert!(parts.is_empty());
    }

    #[test]
    fn split_fixed_strings_partial_last_chunk_is_included() {
        // 20 bytes with width 16: one full chunk + one 4-byte remainder
        let data = b"DEPTH           ABCD";
        let parts = split_fixed_strings(data, 16);
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[1], "ABCD");
    }

    // -----------------------------------------------------------------------
    // read_u32s
    // -----------------------------------------------------------------------

    #[test]
    fn read_u32s_big_endian() {
        let data: Vec<u8> = [1u32, 2, 3].iter().flat_map(|v| v.to_be_bytes()).collect();
        assert_eq!(read_u32s(&data, Endian::Big), vec![1, 2, 3]);
    }

    #[test]
    fn read_u32s_little_endian() {
        let data: Vec<u8> = [1u32, 2, 3].iter().flat_map(|v| v.to_le_bytes()).collect();
        assert_eq!(read_u32s(&data, Endian::Little), vec![1, 2, 3]);
    }

    #[test]
    fn read_u32s_ignores_trailing_incomplete_bytes() {
        // 9 bytes: 2 full u32 + 1 leftover byte, chunks_exact discards the tail
        let mut data: Vec<u8> = [1u32, 2].iter().flat_map(|v| v.to_be_bytes()).collect();
        data.push(0xFF);
        assert_eq!(read_u32s(&data, Endian::Big), vec![1, 2]);
    }

    #[test]
    fn read_u32s_empty_input_gives_empty_vec() {
        assert!(read_u32s(&[], Endian::Big).is_empty());
    }

    // -----------------------------------------------------------------------
    // read_f32s
    // -----------------------------------------------------------------------

    #[test]
    fn read_f32s_big_endian() {
        let data: Vec<u8> = [1.0f32, 2.5, -3.0]
            .iter()
            .flat_map(|v| v.to_be_bytes())
            .collect();
        let result = read_f32s(&data, Endian::Big);
        assert!((result[0] - 1.0f32).abs() < 1e-6);
        assert!((result[1] - 2.5f32).abs() < 1e-6);
        assert!((result[2] - (-3.0f32)).abs() < 1e-6);
    }

    #[test]
    fn read_f32s_little_endian() {
        let data: Vec<u8> = [1.0f32, 2.5].iter().flat_map(|v| v.to_le_bytes()).collect();
        let result = read_f32s(&data, Endian::Little);
        assert!((result[0] - 1.0f32).abs() < 1e-6);
        assert!((result[1] - 2.5f32).abs() < 1e-6);
    }

    #[test]
    fn read_f32s_empty_input_gives_empty_vec() {
        assert!(read_f32s(&[], Endian::Big).is_empty());
    }

    // -----------------------------------------------------------------------
    // read_f64s
    // -----------------------------------------------------------------------

    #[test]
    fn read_f64s_big_endian() {
        let data: Vec<u8> = [1.0f64, 2.5, -3.0]
            .iter()
            .flat_map(|v| v.to_be_bytes())
            .collect();
        let result = read_f64s(&data, Endian::Big);
        assert!((result[0] - 1.0f64).abs() < 1e-12);
        assert!((result[1] - 2.5f64).abs() < 1e-12);
        assert!((result[2] - (-3.0f64)).abs() < 1e-12);
    }

    #[test]
    fn read_f64s_little_endian() {
        let data: Vec<u8> = [1.0f64, 2.5].iter().flat_map(|v| v.to_le_bytes()).collect();
        let result = read_f64s(&data, Endian::Little);
        assert!((result[0] - 1.0f64).abs() < 1e-12);
        assert!((result[1] - 2.5f64).abs() < 1e-12);
    }

    #[test]
    fn read_f64s_empty_input_gives_empty_vec() {
        assert!(read_f64s(&[], Endian::Big).is_empty());
    }

    // -----------------------------------------------------------------------
    // parse_datetime
    // -----------------------------------------------------------------------

    #[test]
    fn parse_datetime_valid_values() {
        let vals = [1972, 7, 13, 17, 15, 13];
        let dt = parse_datetime(&vals, 0).unwrap();
        assert_eq!(format!("{}", dt.date().format("%Y-%m-%d")), "1972-07-13");
        assert_eq!(format!("{}", dt.time().format("%H:%M:%S")), "17:15:13");
    }

    #[test]
    fn parse_datetime_midnight_is_valid() {
        let vals = [2000, 1, 1, 0, 0, 0];
        assert!(parse_datetime(&vals, 0).is_ok());
    }

    #[test]
    fn parse_datetime_end_of_day_is_valid() {
        let vals = [2000, 12, 31, 23, 59, 59];
        assert!(parse_datetime(&vals, 0).is_ok());
    }

    #[test]
    fn parse_datetime_invalid_month_returns_error() {
        let vals = [2000, 13, 1, 0, 0, 0]; // month 13
        assert!(parse_datetime(&vals, 0).is_err());
    }

    #[test]
    fn parse_datetime_invalid_day_returns_error() {
        let vals = [2000, 2, 30, 0, 0, 0]; // Feb 30 never exists
        assert!(parse_datetime(&vals, 0).is_err());
    }

    #[test]
    fn parse_datetime_invalid_hour_returns_error() {
        let vals = [2000, 1, 1, 24, 0, 0]; // hour 24
        assert!(parse_datetime(&vals, 0).is_err());
    }

    #[test]
    fn parse_datetime_invalid_minute_returns_error() {
        let vals = [2000, 1, 1, 0, 60, 0]; // minute 60
        assert!(parse_datetime(&vals, 0).is_err());
    }

    #[test]
    fn parse_datetime_invalid_second_returns_error() {
        let vals = [2000, 1, 1, 0, 0, 60]; // second 60
        assert!(parse_datetime(&vals, 0).is_err());
    }

    #[test]
    fn parse_datetime_month_zero_returns_error() {
        let vals = [2000, 0, 1, 0, 0, 0]; // month 0
        assert!(parse_datetime(&vals, 0).is_err());
    }

    // -----------------------------------------------------------------------
    // read_record
    // -----------------------------------------------------------------------

    #[test]
    fn read_record_valid_little_endian() {
        let payload = b"HELLO";
        let buf = make_record(payload, Endian::Little);
        let mut cursor = Cursor::new(buf);
        let result = read_record(&mut cursor, Endian::Little).unwrap();
        assert_eq!(result, payload);
    }

    #[test]
    fn read_record_valid_big_endian() {
        let payload = b"WORLD";
        let buf = make_record(payload, Endian::Big);
        let mut cursor = Cursor::new(buf);
        let result = read_record(&mut cursor, Endian::Big).unwrap();
        assert_eq!(result, payload);
    }

    #[test]
    fn read_record_empty_payload() {
        let buf = make_record(b"", Endian::Little);
        let mut cursor = Cursor::new(buf);
        let result = read_record(&mut cursor, Endian::Little).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn read_record_mismatched_trailer_returns_error() {
        let payload = b"DATA";
        let len = payload.len() as u32;
        let wrong_len = len + 1;
        let mut buf = Vec::new();
        buf.extend_from_slice(&len.to_le_bytes());
        buf.extend_from_slice(payload);
        buf.extend_from_slice(&wrong_len.to_le_bytes()); // trailer != header
        let mut cursor = Cursor::new(buf);
        assert!(read_record(&mut cursor, Endian::Little).is_err());
    }

    #[test]
    fn read_record_wrong_endian_returns_error() {
        // Build a valid LE record, try to read it as BE — length mismatch expected
        let payload = vec![0u8; 80];
        let buf = make_record(&payload, Endian::Little);
        let mut cursor = Cursor::new(buf);
        // Reading as BE will interpret the length bytes incorrectly, likely failing
        assert!(read_record(&mut cursor, Endian::Big).is_err());
    }

    #[test]
    fn read_record_advances_cursor_to_end_of_record() {
        let payload = b"ABCDE";
        let buf = make_record(payload, Endian::Little);
        let total_len = buf.len() as u64;
        let mut cursor = Cursor::new(buf);
        read_record(&mut cursor, Endian::Little).unwrap();
        assert_eq!(cursor.position(), total_len);
    }

    // -----------------------------------------------------------------------
    // detect_endianness
    // -----------------------------------------------------------------------

    #[test]
    fn detect_endianness_little_endian_file() {
        let buf = make_record(
            b"SELAFIN TITLE                                                           ",
            Endian::Little,
        );
        let mut cursor = Cursor::new(buf);
        let endian = detect_endianness(&mut cursor).unwrap();
        assert_eq!(endian, Endian::Little);
    }

    #[test]
    fn detect_endianness_big_endian_file() {
        let buf = make_record(
            b"SELAFIN TITLE                                                           ",
            Endian::Big,
        );
        let mut cursor = Cursor::new(buf);
        let endian = detect_endianness(&mut cursor).unwrap();
        assert_eq!(endian, Endian::Big);
    }

    #[test]
    fn detect_endianness_rewinds_cursor_to_start() {
        let buf = make_record(
            b"SOME TITLE                                                              ",
            Endian::Little,
        );
        let mut cursor = Cursor::new(buf);
        detect_endianness(&mut cursor).unwrap();
        // After detection the cursor must be back at 0 so the caller can re-read
        assert_eq!(cursor.position(), 0);
    }

    #[test]
    fn detect_endianness_invalid_data_returns_error() {
        // Pure garbage: no valid Fortran record possible in either endianness
        let buf = vec![0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x01];
        let mut cursor = Cursor::new(buf);
        assert!(detect_endianness(&mut cursor).is_err());
    }

    // -----------------------------------------------------------------------
    // detect_float_size
    // -----------------------------------------------------------------------

    #[test]
    fn detect_float_size_f32_little_endian() {
        let npoin3 = 4usize;
        let payload: Vec<u8> = (0..npoin3)
            .flat_map(|i| f32_bytes(i as f32, Endian::Little))
            .collect();
        let buf = make_record(&payload, Endian::Little);
        let mut cursor = Cursor::new(buf);
        let result = detect_float_size(&mut cursor, Endian::Little, npoin3).unwrap();
        assert_eq!(result, FloatSize::F32);
    }

    #[test]
    fn detect_float_size_f64_little_endian() {
        let npoin3 = 4usize;
        let payload: Vec<u8> = (0..npoin3)
            .flat_map(|i| f64_bytes(i as f64, Endian::Little))
            .collect();
        let buf = make_record(&payload, Endian::Little);
        let mut cursor = Cursor::new(buf);
        let result = detect_float_size(&mut cursor, Endian::Little, npoin3).unwrap();
        assert_eq!(result, FloatSize::F64);
    }

    #[test]
    fn detect_float_size_f32_big_endian() {
        let npoin3 = 3usize;
        let payload: Vec<u8> = (0..npoin3)
            .flat_map(|i| f32_bytes(i as f32, Endian::Big))
            .collect();
        let buf = make_record(&payload, Endian::Big);
        let mut cursor = Cursor::new(buf);
        let result = detect_float_size(&mut cursor, Endian::Big, npoin3).unwrap();
        assert_eq!(result, FloatSize::F32);
    }

    #[test]
    fn detect_float_size_f64_big_endian() {
        let npoin3 = 3usize;
        let payload: Vec<u8> = (0..npoin3)
            .flat_map(|i| f64_bytes(i as f64, Endian::Big))
            .collect();
        let buf = make_record(&payload, Endian::Big);
        let mut cursor = Cursor::new(buf);
        let result = detect_float_size(&mut cursor, Endian::Big, npoin3).unwrap();
        assert_eq!(result, FloatSize::F64);
    }

    #[test]
    fn detect_float_size_rewinds_cursor_to_start() {
        let npoin3 = 2usize;
        let payload: Vec<u8> = (0..npoin3)
            .flat_map(|i| f32_bytes(i as f32, Endian::Little))
            .collect();
        let buf = make_record(&payload, Endian::Little);
        let mut cursor = Cursor::new(buf);
        detect_float_size(&mut cursor, Endian::Little, npoin3).unwrap();
        // Must rewind so the caller can consume the record normally
        assert_eq!(cursor.position(), 0);
    }

    #[test]
    fn detect_float_size_wrong_npoin3_returns_error() {
        // Build an f32 record for 4 points but tell the function to expect 5
        let npoin3 = 4usize;
        let payload: Vec<u8> = (0..npoin3)
            .flat_map(|i| f32_bytes(i as f32, Endian::Little))
            .collect();
        let buf = make_record(&payload, Endian::Little);
        let mut cursor = Cursor::new(buf);
        assert!(detect_float_size(&mut cursor, Endian::Little, npoin3 + 1).is_err());
    }

    // -----------------------------------------------------------------------
    // read_variable_records
    // -----------------------------------------------------------------------

    fn make_variable_record(name: &str, unit: &str, endian: Endian) -> Vec<u8> {
        let mut payload = [b' '; 32];
        let name_bytes = name.as_bytes();
        let unit_bytes = unit.as_bytes();
        payload[..name_bytes.len().min(16)]
            .copy_from_slice(&name_bytes[..name_bytes.len().min(16)]);
        payload[16..16 + unit_bytes.len().min(16)]
            .copy_from_slice(&unit_bytes[..unit_bytes.len().min(16)]);
        make_record(&payload, endian)
    }

    #[test]
    fn read_variable_records_parses_name_and_unit() {
        let buf = make_variable_record("VELOCITY U", "M/S", Endian::Little);
        let mut cursor = Cursor::new(buf);
        let vars = read_variable_records(&mut cursor, Endian::Little, 1).unwrap();
        assert_eq!(vars.len(), 1);
        assert_eq!(vars[0].name, "VELOCITY U");
        assert_eq!(vars[0].unit, "M/S");
    }

    #[test]
    fn read_variable_records_parses_multiple_variables() {
        let mut buf = Vec::new();
        buf.extend(make_variable_record("DEPTH", "M", Endian::Little));
        buf.extend(make_variable_record("VELOCITY U", "M/S", Endian::Little));
        buf.extend(make_variable_record("VELOCITY V", "M/S", Endian::Little));
        let mut cursor = Cursor::new(buf);
        let vars = read_variable_records(&mut cursor, Endian::Little, 3).unwrap();
        assert_eq!(vars.len(), 3);
        assert_eq!(vars[0].name, "DEPTH");
        assert_eq!(vars[1].name, "VELOCITY U");
        assert_eq!(vars[2].name, "VELOCITY V");
    }

    #[test]
    fn read_variable_records_zero_count_returns_empty_vec() {
        let mut cursor = Cursor::new(vec![]);
        let vars = read_variable_records(&mut cursor, Endian::Little, 0).unwrap();
        assert!(vars.is_empty());
    }

    #[test]
    fn read_variable_records_trims_trailing_spaces() {
        let buf = make_variable_record("DEPTH           ", "M               ", Endian::Little);
        let mut cursor = Cursor::new(buf);
        let vars = read_variable_records(&mut cursor, Endian::Little, 1).unwrap();
        assert_eq!(vars[0].name, "DEPTH");
        assert_eq!(vars[0].unit, "M");
    }

    #[test]
    fn read_variable_records_too_short_record_returns_error() {
        // A record with only 16 bytes instead of the required 32
        let payload = b"DEPTH           "; // only 16 bytes
        let buf = make_record(payload, Endian::Little);
        let mut cursor = Cursor::new(buf);
        assert!(read_variable_records(&mut cursor, Endian::Little, 1).is_err());
    }
}

// cSpell:ignore SELAFIND SELAPHIN nvar ncld KNOLG IPOBO vals
