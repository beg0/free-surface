//! # Selafin Parser
//!
//! Parses the Selafin binary file format.
//! Supports both big-endian and little-endian files, and both f32 and f64 meshes.

use super::{Selafin, SlfMesh, SlfVariable};
use binrw::{BinReaderExt, Endian};
use chrono::NaiveDateTime;
use std::cmp::max;
use std::io::{Read, Seek, SeekFrom};

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

#[derive(Clone, Copy, Debug, PartialEq)]
enum FloatSize {
    F32,
    F64,
}

/// After all integer geometry records have been consumed, the next record
/// contains `npoin3` floats.  We know npoin3, so we can check which size fits.
fn detect_float_size<R: Read + Seek>(
    reader: &mut R,
    endian: Endian,
    npoin3: u32,
) -> binrw::BinResult<FloatSize> {
    let start = reader.stream_position()?;

    let data = read_record(reader, endian)?;
    reader.seek(SeekFrom::Start(start))?;

    let expected_f32 = npoin3 as usize * 4;
    let expected_f64 = npoin3 as usize * 8;

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
/// ```rust
/// use std::fs::File;
/// use std::io::BufReader;
///
/// let f = File::open("my_file.slf").unwrap();
/// let selafin = parser::parse(BufReader::new(f)).unwrap();
/// ```
pub fn parse<R: Read + Seek>(mut reader: R) -> binrw::BinResult<Selafin> {
    // -----------------------------------------------------------------------
    // 1. Detect endianness
    // -----------------------------------------------------------------------
    let endian = detect_endianness(&mut reader)?;

    let mut slf = Selafin::default();

    // -----------------------------------------------------------------------
    // 2. Metadata
    // -----------------------------------------------------------------------

    // 1.1 - Title (80-character string)
    {
        let data = read_record(&mut reader, endian)?;
        if data.len() < 80 {
            return Err(binrw::Error::AssertFail {
                pos: reader.stream_position()?,
                message: format!("Title record too short: {} bytes", data.len()),
            });
        }
        slf.title = ascii_record_to_string(&data[..80]);
    }

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
    slf.var = read_variable_records(&mut reader, endian, nvar)?;

    // 1.4 - `cld` entries
    slf.cld = read_variable_records(&mut reader, endian, ncld)?;

    let iparams: Vec<u32>;
    // 1.5 - iparam (10 × i32)
    {
        let data = read_record(&mut reader, endian)?;
        iparams = read_u32s(&data, endian);
        if iparams.len() != 10 {
            return Err(binrw::Error::AssertFail {
                pos: reader.stream_position()?,
                message: format!("iparam record has {} values, expected 10", iparams.len()),
            });
        }
        //slf.iparam.copy_from_slice(&iparams[..10]);
    }
    slf.origin = (
        iparams[IParams::XOrigin as usize],
        iparams[IParams::YOrigin as usize],
    );
    slf.boundaries_count = iparams[IParams::BoundariesCnt as usize];
    slf.interfaces_count = iparams[IParams::IfaceCnt as usize];

    // 1.6 - Optional datetime record when iparam[9] == 1
    if iparams[IParams::HasDateTime as usize] == 1 {
        let data = read_record(&mut reader, endian)?;
        let vals = read_u32s(&data, endian);
        if vals.len() < 6 {
            return Err(binrw::Error::AssertFail {
                pos: reader.stream_position()?,
                message: format!("datetime record has only {} values, expected 6", vals.len()),
            });
        }

        slf.datetime = parse_datetime(&vals[..6], reader.stream_position()?)?;
    }

    // -----------------------------------------------------------------------
    // 3. Geometry - integer elements
    // -----------------------------------------------------------------------

    // 2.1 - (nelem3, npoin3, npd3, nplan)
    {
        let data = read_record(&mut reader, endian)?;
        let vals = read_u32s(&data, endian);
        if vals.len() < 4 {
            return Err(binrw::Error::AssertFail {
                pos: reader.stream_position()?,
                message: format!("Geometry integer header has only {} values", vals.len()),
            });
        }
        slf.nelem3 = vals[0];
        slf.npoin3 = vals[1];
        slf.npd3 = vals[2];
        slf.nplan = max(vals[3], 1);

        if slf.nplan != max(iparams[IParams::PlanesCnt as usize], 1) {
            return Err(binrw::Error::AssertFail {
                pos: reader.stream_position()?,
                message: format!(
                    "Inconsistent number of planes, IPARAMS says {}, Geometry says {}",
                    iparams[IParams::PlanesCnt as usize],
                    slf.nplan
                ),
            });
        }

        // Derive 2-D values
        if slf.nplan > 1 {
            slf.nelem2 = slf.nelem3 / (slf.nplan - 1);
            slf.npoin2 = slf.npoin3 / slf.nplan;
            slf.npd2 = slf.npd3 as i32 / 2; // triangular prism: 6 nodes → 3 in 2-D
        } else {
            slf.nelem2 = slf.nelem3;
            slf.npoin2 = slf.npoin3;
            slf.npd2 = slf.npd3 as i32;
        }
    }

    // 2.2 - ikle3 (nelem3 × npd3 connectivity table)
    {
        let expected = (slf.nelem3 * slf.npd3) as usize;
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
        slf.ikle3 = vals;
    }

    // 2.3 - ipob3 (npoin3 boundary codes)
    // TODO: if iparams[7] or iparams[8] != 0, then it's KNOLG and not IPOBO
    {
        let expected = slf.npoin3 as usize;
        let data = read_record(&mut reader, endian)?;
        let vals = read_i32s(&data, endian);
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
        slf.ipob3 = vals;
    }

    // Derive 2-D connectivity / boundary arrays from 3-D equivalents when 3-D
    if slf.nplan > 1 {
        let npd2 = slf.npd2 as usize;
        //let npd3 = slf.npd3 as usize;
        let nelem2 = slf.nelem2 as usize;
        // Bottom layer of ikle3 gives ikle2
        slf.ikle2 = slf.ikle3[..nelem2 * npd2].to_vec();
        // Bottom layer of ipob3 gives ipob2
        let npoin2 = slf.npoin2 as usize;
        slf.ipob2 = slf.ipob3[..npoin2].to_vec();
    } else {
        slf.ikle2 = slf.ikle3.clone();
        slf.ipob2 = slf.ipob3.clone();
    }

    // -----------------------------------------------------------------------
    // 4. Geometry - float mesh (detect f32 vs f64 here)
    // -----------------------------------------------------------------------
    let float_size = detect_float_size(&mut reader, endian, slf.npoin3)?;

    let mesh_x_data = read_record(&mut reader, endian)?;
    let mesh_y_data = read_record(&mut reader, endian)?;

    slf.mesh = match float_size {
        FloatSize::F32 => SlfMesh::Float {
            x: read_f32s(&mesh_x_data, endian),
            y: read_f32s(&mesh_y_data, endian),
        },
        FloatSize::F64 => SlfMesh::Double {
            x: read_f64s(&mesh_x_data, endian),
            y: read_f64s(&mesh_y_data, endian),
        },
    };

    // -----------------------------------------------------------------------
    // 5. Optional time history (remaining records, each npoin3 floats)
    //
    // The caller may retrieve these separately; we skip them here and leave
    // the reader positioned at the first time-step record (if any).
    // Uncomment the block below to collect all time-step data eagerly.
    // -----------------------------------------------------------------------
    //
    // let nvar_total = slf.nbvar();
    // loop {
    //     read_record(&mut reader, endian)
    //     match read_record(&mut reader, endian) {
    //         Ok(data) => { /* store according to float_size */ }
    //         Err(_) => break, // EOF or short record → done
    //     }
    // }

    Ok(slf)
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
