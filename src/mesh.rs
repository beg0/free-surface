//! # Mesh operations
//!

pub mod celliterator;
pub mod neighbors;
pub mod percellcoords;

mod dtm;

pub use dtm::{init_dtm, DTM};
