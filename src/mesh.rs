//! # Mesh operations
//!

pub mod celliterator;
pub mod elementtype;
pub mod neighbors;
pub mod percellcoords;

mod dtm;
mod sanity;

pub use dtm::{init_dtm, DTM};
