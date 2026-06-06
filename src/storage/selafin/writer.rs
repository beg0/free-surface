//! # Selafin Writer
//!
//! Serializes a [`Selafin`] structure into the Selafin binary file format.
//!
//! Endianness is chosen by the caller; the float size (f32 vs f64) is derived
//! automatically from the mesh stored in the [`Selafin`] (i.e. a file parsed
//! as f64 is written back as f64).
//!
//! # Example
//! ```no_run
//! use std::fs::File;
//! use std::io::BufWriter;
//! use binrw::Endian;
//! use free_surface::storage::selafin::{Selafin, parse_file, write};
//!
//! let slf: Selafin = parse_file("input.slf").unwrap();
//! let out = File::create("output.slf").unwrap();
//! write(BufWriter::new(out), &slf, Endian::Little).unwrap();
//! ```

use super::container::{SlfArray1D, SlfArray2D};
use super::variable::SlfVariable;
use super::Selafin;
use binrw::Endian;
use std::io::{Result, Write};

// ---------------------------------------------------------------------------
// Low-level write helpers
// ---------------------------------------------------------------------------

/// Write a single `u32` in the requested endianness.
#[inline]
fn write_u32<W: Write>(w: &mut W, v: u32, endian: Endian) -> Result<()> {
    let bytes = match endian {
        Endian::Big => v.to_be_bytes(),
        Endian::Little => v.to_le_bytes(),
    };
    w.write_all(&bytes)
}

/// Wrap `payload` in a Fortran-style record:
///   u32 length | payload bytes | u32 length
fn write_record<W: Write>(w: &mut W, payload: &[u8], endian: Endian) -> Result<()> {
    let len = payload.len() as u32;
    write_u32(w, len, endian)?;
    w.write_all(payload)?;
    write_u32(w, len, endian)
}

/// Encode a `u32` slice to bytes.
fn u32s_to_bytes(values: &[u32], endian: Endian) -> Vec<u8> {
    values
        .iter()
        .flat_map(|&v| match endian {
            Endian::Big => v.to_be_bytes(),
            Endian::Little => v.to_le_bytes(),
        })
        .collect()
}

/// Encode an `f32` slice to bytes.
fn f32s_to_bytes(values: &[f32], endian: Endian) -> Vec<u8> {
    values
        .iter()
        .flat_map(|&v| match endian {
            Endian::Big => v.to_be_bytes(),
            Endian::Little => v.to_le_bytes(),
        })
        .collect()
}

/// Encode an `f64` slice to bytes.
fn f64s_to_bytes(values: &[f64], endian: Endian) -> Vec<u8> {
    values
        .iter()
        .flat_map(|&v| match endian {
            Endian::Big => v.to_be_bytes(),
            Endian::Little => v.to_le_bytes(),
        })
        .collect()
}

/// Encode a `SlfArray1D` to bytes, using the given endianness.
fn array1d_to_bytes(arr: &SlfArray1D, endian: Endian) -> Vec<u8> {
    match arr {
        SlfArray1D::Float(v) => f32s_to_bytes(v, endian),
        SlfArray1D::Double(v) => f64s_to_bytes(v, endian),
    }
}

// ---------------------------------------------------------------------------
// Fixed-width string helpers
// ---------------------------------------------------------------------------

/// Write `s` into a fixed-width field of `width` bytes, space-padded on the
/// right.  Silently truncates if `s` is longer than `width`.
fn fixed_str(s: &str, width: usize) -> Vec<u8> {
    let mut buf = vec![b' '; width];
    let src = s.as_bytes();
    let copy_len = src.len().min(width);
    buf[..copy_len].copy_from_slice(&src[..copy_len]);
    buf
}

// ---------------------------------------------------------------------------
// Section writers
// ---------------------------------------------------------------------------

/// 1.1 - Title (80-byte string record, then 8 bytes of type tag also padded).
///
/// The Selafin standard uses an 80-byte title followed immediately by 8 bytes
/// for the file type tag (e.g. `"SERAFIN "` or `"SERAFIND"`).  Both are
/// packed into a single 88-byte record.
fn write_title<W: Write>(w: &mut W, slf: &Selafin, endian: Endian) -> Result<()> {
    let float_tag: &[u8] = match slf.geometry().points_raw() {
        SlfArray2D::Float { .. } => b"SERAFIN ",
        SlfArray2D::Double { .. } => b"SERAFIND",
    };

    let title = slf.title();
    let payload = if title.len() <= 72 {
        let mut padded = fixed_str(title, 72);
        padded.extend_from_slice(float_tag);
        padded
    } else {
        fixed_str(title, 80)
    };

    write_record(w, &payload, endian)
}

/// 1.2 - Variable counts (nvar, ncld) as a 2 × u32 record.
fn write_var_counts<W: Write>(w: &mut W, slf: &Selafin, endian: Endian) -> Result<()> {
    let mut payload = Vec::with_capacity(8);
    payload.extend_from_slice(&u32s_to_bytes(&[slf.nbvar1() as u32], endian));
    payload.extend_from_slice(&u32s_to_bytes(&[slf.nbvar2() as u32], endian));
    write_record(w, &payload, endian)
}

/// 1.3 / 1.4 - Variable name+unit records (one 32-byte record per variable).
fn write_variables<W: Write>(w: &mut W, vars: &[SlfVariable], endian: Endian) -> Result<()> {
    for v in vars {
        let mut payload = fixed_str(&v.name, 16);
        payload.extend_from_slice(&fixed_str(&v.unit, 16));
        write_record(w, &payload, endian)?;
    }
    Ok(())
}

/// 1.5 - iparam block (10 × u32).
///
/// Most values come from the geometry; we reconstruct the same layout the
/// parser expects so that a round-trip is lossless.
fn write_iparam<W: Write>(w: &mut W, slf: &Selafin, endian: Endian) -> Result<()> {
    let geo = slf.geometry();
    let mut iparam = [0u32; 10];

    // Indices match the `IParams` enum in parser.rs
    iparam[2] = slf.origin.0; // XOrigin
    iparam[3] = slf.origin.1; // YOrigin
    iparam[6] = geo.planes_cnt(); // PlanesCnt
    iparam[7] = geo.boundaries_count; // BoundariesCnt
    iparam[8] = geo.interfaces_count; // IfaceCnt
    iparam[9] = if slf.datetime.is_some() { 1 } else { 0 }; // HasDateTime

    write_record(w, &u32s_to_bytes(&iparam, endian), endian)
}

/// 1.6 - Optional datetime record (6 × u32: YYYY MM DD HH MM SS).
fn write_datetime<W: Write>(w: &mut W, slf: &Selafin, endian: Endian) -> Result<()> {
    if let Some(dt) = slf.datetime {
        use chrono::Datelike as _;
        use chrono::Timelike as _;
        let vals = [
            dt.year() as u32,
            dt.month(),
            dt.day(),
            dt.hour(),
            dt.minute(),
            dt.second(),
        ];
        write_record(w, &u32s_to_bytes(&vals, endian), endian)?;
    }
    Ok(())
}

/// 2.1 - Geometry integer header (nelem3, npoin3, npd3, nplan).
fn write_geometry_header<W: Write>(w: &mut W, slf: &Selafin, endian: Endian) -> Result<()> {
    let geo = slf.geometry();
    let vals = [
        geo.elements_count() as u32,
        geo.points_count() as u32,
        geo.point_per_element() as u32,
        geo.planes_cnt(),
    ];
    write_record(w, &u32s_to_bytes(&vals, endian), endian)
}

/// 2.2 - Connectivity table ikle3.
///
/// The parser converts from 1-based to 0-based on read; we convert back here.
fn write_ikle3<W: Write>(w: &mut W, slf: &Selafin, endian: Endian) -> Result<()> {
    let one_based: Vec<u32> = slf.geometry().ikle3().iter().map(|&v| v + 1).collect();
    write_record(w, &u32s_to_bytes(&one_based, endian), endian)
}

/// 2.3 - Boundary node table ipob3.
///
/// The parser filters out inner nodes (value 0) and converts to 0-based.
/// We reconstruct the full npoin3-length array: inner nodes → 0, boundary
/// nodes → 1-based index.
fn write_ipob3<W: Write>(w: &mut W, slf: &Selafin, endian: Endian) -> Result<()> {
    let geo = slf.geometry();
    let npoin3 = geo.points_count();

    // // Build a set of boundary node indices for O(1) lookup
    // let boundary_set: std::collections::HashSet<u32> =
    //     geo.ipob3().iter().cloned().collect();

    // // For each point, emit its 1-based boundary index or 0 for inner nodes.
    // // We use the position in ipob3_raw as the 1-based value for boundary nodes.
    // let mut ipob3_map = vec![0u32; npoin3];
    // for (rank, &node_idx) in geo.ipob3().iter().enumerate() {
    //     if (node_idx as usize) < npoin3 {
    //         ipob3_map[node_idx as usize] = (rank + 1) as u32;
    //     }
    // }

    let mut ipob3_map: Vec<u32> = geo.ipob3().iter().map(|v| *v + 1).collect();
    let rem_size = npoin3 - ipob3_map.len();
    if rem_size > 0 {
        let mut padding = vec![0_u32; rem_size];
        ipob3_map.append(&mut padding);
    }

    write_record(w, &u32s_to_bytes(&ipob3_map, endian), endian)
}

/// 3.1 / 3.2 - Mesh coordinate records (x then y).
fn write_mesh<W: Write>(w: &mut W, slf: &Selafin, endian: Endian) -> Result<()> {
    match slf.geometry().points_raw() {
        SlfArray2D::Float { x, y } => {
            write_record(w, &f32s_to_bytes(x, endian), endian)?;
            write_record(w, &f32s_to_bytes(y, endian), endian)?;
        }
        SlfArray2D::Double { x, y } => {
            write_record(w, &f64s_to_bytes(x, endian), endian)?;
            write_record(w, &f64s_to_bytes(y, endian), endian)?;
        }
    }
    Ok(())
}

/// 4 - Time history: for each time step, one time record then one record per
/// variable, in the same order as the variable definitions (var then cld).
fn write_history<W: Write>(w: &mut W, slf: &Selafin, endian: Endian) -> Result<()> {
    let ts = slf.results();

    if ts.is_empty() {
        return Ok(());
    }

    // Build the ordered variable name list (var first, then cld) so we write
    // time steps in the same order the parser expects.
    let var_names: Vec<&str> = slf
        .var_defs()
        .iter()
        .chain(slf.cld_defs().iter())
        .map(|v| v.name.as_str())
        .collect();

    let step_count = ts.step_count();
    let time = ts.time();

    for step in 0..step_count {
        // Time value for this step — slice a single element from the time array
        let time_payload = match time {
            SlfArray1D::Float(v) => f32s_to_bytes(&v[step..=step], endian),
            SlfArray1D::Double(v) => f64s_to_bytes(&v[step..=step], endian),
        };
        write_record(w, &time_payload, endian)?;

        // One record per variable
        for &name in &var_names {
            if let Some(ve) = ts.get_var(name) {
                write_record(w, &array1d_to_bytes(&ve.values[step], endian), endian)?;
            }
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Serialize `slf` to `writer` using the given endianness.
///
/// The float size (f32 vs f64) is determined by the mesh stored in `slf`:
/// - `SlfArray2D::Float`  → all float records written as `f32`
/// - `SlfArray2D::Double` → all float records written as `f64`
pub fn write<W: Write>(mut w: W, slf: &Selafin, endian: Endian) -> Result<()> {
    write_title(&mut w, slf, endian)?;
    write_var_counts(&mut w, slf, endian)?;
    write_variables(&mut w, slf.var_defs(), endian)?;
    write_variables(&mut w, slf.cld_defs(), endian)?;
    write_iparam(&mut w, slf, endian)?;
    write_datetime(&mut w, slf, endian)?;
    write_geometry_header(&mut w, slf, endian)?;
    write_ikle3(&mut w, slf, endian)?;
    write_ipob3(&mut w, slf, endian)?;
    write_mesh(&mut w, slf, endian)?;
    write_history(&mut w, slf, endian)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Convenience: write to a file by path
// ---------------------------------------------------------------------------

use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

/// Serialize `slf` to the file at `path`, creating or truncating it.
pub fn write_file<P: AsRef<Path>>(path: P, slf: &Selafin, endian: Endian) -> Result<()> {
    let file = File::create(path)?;
    write(BufWriter::new(file), slf, endian)
}

// cSpell:ignore nvar ncld vals
