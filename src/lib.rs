//! # Free Surface
//!
//! Free surface is a reliable & fast Computational Fluid Dynamics (CFD) tool.
//! It is designed to be:
//!  - easy to use
//!  - compatible with existing tools
//!  - fast
//!  - robust to errors
//!

pub mod aui;
pub mod config;
pub mod i18n;
pub mod math;
pub mod mesh;
pub mod storage;
pub mod utils;

pub use i18n::set_locale;
