//! # Variables and results in a selafin file
//!

/// A Selafin variable
#[derive(Debug, Clone, Default)]
pub struct SlfVariable {
    #[allow(dead_code)]
    pub name: String,
    #[allow(dead_code)]
    pub unit: String,
}

impl SlfVariable {
    pub fn new(name: &str, unit: &str) -> Self {
        SlfVariable {
            name: name.to_string(),
            unit: unit.to_string(),
        }
    }
}
