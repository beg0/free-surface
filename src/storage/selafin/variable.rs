//! # Variables and results in a selafin file
//!
use super::container::SlfArray1D;
use std::collections::HashMap;

/// A Selafin variable
#[derive(Debug, Clone, Default)]
pub struct SlfVariable {
    pub name: String,
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

/// Evolution of a time-variable over a mesh
#[derive(Debug)]
pub struct VariableEvolution {
    pub var: SlfVariable,

    // For each time stamp, values of each point in the mesh
    pub values: Vec<SlfArray1D>,
}

/// Evolution of several time-variables over a mesh

#[derive(Debug, Default)]
pub struct TimeSerie {
    time: SlfArray1D,
    vars: HashMap<String, VariableEvolution>,
}

impl TimeSerie {
    pub fn new(time: SlfArray1D, vars: HashMap<String, VariableEvolution>) -> Self {
        let step_count = time.len();

        for v in vars.values() {
            assert_eq!(v.values.len(), step_count);
        }

        Self { time, vars }
    }
    pub fn is_empty(&self) -> bool {
        self.time.is_empty() || self.vars.is_empty()
    }

    pub fn step_count(&self) -> usize {
        self.time.len()
    }

    pub fn var_count(&self) -> usize {
        self.vars.len()
    }

    /// The time axis as a slice-like reference to the underlying SlfArray1D.
    pub fn time(&self) -> &SlfArray1D {
        &self.time
    }

    /// Iterate over all variable evolutions in insertion-independent order.
    /// Yields `(&name, &VariableEvolution)` pairs.
    pub fn iter_vars(&self) -> impl Iterator<Item = (&String, &VariableEvolution)> {
        self.vars.iter()
    }

    /// Look up a single variable evolution by name.
    pub fn get_var(&self, name: &str) -> Option<&VariableEvolution> {
        let upper_name = name.to_uppercase();
        self.vars.get(&upper_name)
    }
}

#[cfg(test)]
mod tests {
    use super::super::container::{FloatSize, SlfArray1D};
    use super::*;
    use std::collections::HashMap;

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    /// Build a time array with `n` evenly-spaced f32 steps starting at 0.
    fn make_time_f32(n: usize) -> SlfArray1D {
        SlfArray1D::Float((0..n).map(|i| i as f32).collect())
    }

    /// Build a time array with `n` evenly-spaced f64 steps starting at 0.
    fn make_time_f64(n: usize) -> SlfArray1D {
        SlfArray1D::Double((0..n).map(|i| i as f64).collect())
    }

    /// Build a `VariableEvolution` with `step_count` snapshots, each holding
    /// `mesh_size` f32 values equal to `fill`.
    fn make_var_f32(
        name: &str,
        unit: &str,
        step_count: usize,
        mesh_size: usize,
        fill: f32,
    ) -> VariableEvolution {
        VariableEvolution {
            var: SlfVariable::new(name, unit),
            values: (0..step_count)
                .map(|_| SlfArray1D::Float(vec![fill; mesh_size]))
                .collect(),
        }
    }

    /// Same as above but f64.
    fn make_var_f64(
        name: &str,
        unit: &str,
        step_count: usize,
        mesh_size: usize,
        fill: f64,
    ) -> VariableEvolution {
        VariableEvolution {
            var: SlfVariable::new(name, unit),
            values: (0..step_count)
                .map(|_| SlfArray1D::Double(vec![fill; mesh_size]))
                .collect(),
        }
    }
    // -----------------------------------------------------------------------
    // SlfVariable
    // -----------------------------------------------------------------------

    #[test]
    fn variable_new_stores_name_and_unit() {
        let v = SlfVariable::new("VELOCITY U", "M/S");
        assert_eq!(v.name, "VELOCITY U");
        assert_eq!(v.unit, "M/S");
    }

    #[test]
    fn variable_default_is_empty_strings() {
        let v = SlfVariable::default();
        assert!(v.name.is_empty());
        assert!(v.unit.is_empty());
    }

    #[test]
    fn variable_clone_is_independent() {
        let original = SlfVariable::new("DEPTH", "M");
        let mut cloned = original.clone();
        cloned.name = "OTHER".to_string();
        assert_eq!(original.name, "DEPTH");
    }

    // -----------------------------------------------------------------------
    // SlfArray1D basics (used heavily by the structs above)
    // -----------------------------------------------------------------------

    #[test]
    fn array1d_len_matches_underlying_vec_f32() {
        let a = SlfArray1D::Float(vec![1.0, 2.0, 3.0]);
        assert_eq!(a.len(), 3);
    }

    #[test]
    fn array1d_len_matches_underlying_vec_f64() {
        let a = SlfArray1D::Double(vec![1.0, 2.0]);
        assert_eq!(a.len(), 2);
    }

    #[test]
    fn array1d_is_empty_on_new() {
        assert!(SlfArray1D::new(FloatSize::F32).is_empty());
        assert!(SlfArray1D::new(FloatSize::F64).is_empty());
    }

    #[test]
    fn array1d_is_not_empty_after_data() {
        let a: SlfArray1D = vec![0.0f32].into();
        assert!(!a.is_empty());
    }

    #[test]
    fn array1d_from_vec_f32_round_trips() {
        let src = vec![1.0f32, 2.0, 3.0];
        let a: SlfArray1D = src.clone().into();
        let out: Vec<f32> = a.into();
        assert_eq!(out, src);
    }

    #[test]
    fn array1d_from_vec_f64_round_trips() {
        let src = vec![1.0f64, 2.0, 3.0];
        let a: SlfArray1D = src.clone().into();
        let out: Vec<f64> = a.into();
        assert_eq!(out, src);
    }

    #[test]
    fn array1d_float_converts_to_f64() {
        let a: SlfArray1D = vec![1.0f32, 2.0].into();
        let out: Vec<f64> = a.into();
        assert!((out[0] - 1.0f64).abs() < 1e-6);
        assert!((out[1] - 2.0f64).abs() < 1e-6);
    }

    #[test]
    fn array1d_double_converts_to_f32() {
        let a: SlfArray1D = vec![1.0f64, 2.0].into();
        let out: Vec<f32> = a.into();
        assert!((out[0] - 1.0f32).abs() < 1e-6);
        assert!((out[1] - 2.0f32).abs() < 1e-6);
    }

    #[test]
    fn array1d_default_is_float_empty() {
        let a = SlfArray1D::default();
        assert!(a.is_empty());
        assert!(matches!(a, SlfArray1D::Float(_)));
    }

    // -----------------------------------------------------------------------
    // TimeSerie — happy path
    // -----------------------------------------------------------------------

    #[test]
    fn timeserie_new_single_var_f32() {
        let time = make_time_f32(3);
        let mut vars = HashMap::new();
        vars.insert("DEPTH".to_string(), make_var_f32("DEPTH", "M", 3, 10, 0.0));

        let ts = TimeSerie::new(time, vars);

        assert_eq!(ts.step_count(), 3);
        assert_eq!(ts.var_count(), 1);
        assert!(!ts.is_empty());
    }

    #[test]
    fn timeserie_new_single_var_f64() {
        let time = make_time_f64(5);
        let mut vars = HashMap::new();
        vars.insert(
            "VELOCITY U".to_string(),
            make_var_f64("VELOCITY U", "M/S", 5, 4, 1.5),
        );

        let ts = TimeSerie::new(time, vars);

        assert_eq!(ts.step_count(), 5);
        assert_eq!(ts.var_count(), 1);
    }

    #[test]
    fn timeserie_new_multiple_vars() {
        let steps = 4;
        let time = make_time_f32(steps);
        let mut vars = HashMap::new();
        vars.insert(
            "DEPTH".to_string(),
            make_var_f32("DEPTH", "M", steps, 6, 0.0),
        );
        vars.insert(
            "VELOCITY U".to_string(),
            make_var_f32("VELOCITY U", "M/S", steps, 6, 1.0),
        );
        vars.insert(
            "VELOCITY V".to_string(),
            make_var_f32("VELOCITY V", "M/S", steps, 6, 2.0),
        );

        let ts = TimeSerie::new(time, vars);

        assert_eq!(ts.step_count(), steps);
        assert_eq!(ts.var_count(), 3);
    }

    #[test]
    fn timeserie_new_single_step() {
        let time = make_time_f32(1);
        let mut vars = HashMap::new();
        vars.insert("DEPTH".to_string(), make_var_f32("DEPTH", "M", 1, 100, 5.0));

        let ts = TimeSerie::new(time, vars);

        assert_eq!(ts.step_count(), 1);
        assert!(!ts.is_empty());
    }

    #[test]
    fn timeserie_new_mixed_float_sizes() {
        // time is f64, variables are f32 — both are valid SlfArray1D variants
        let time = make_time_f64(2);
        let mut vars = HashMap::new();
        vars.insert("DEPTH".to_string(), make_var_f32("DEPTH", "M", 2, 8, 3.0));

        let ts = TimeSerie::new(time, vars);
        assert_eq!(ts.step_count(), 2);
    }

    // -----------------------------------------------------------------------
    // TimeSerie — edge cases
    // -----------------------------------------------------------------------

    #[test]
    fn timeserie_is_empty_with_empty_time() {
        let time = SlfArray1D::Float(vec![]);
        let ts = TimeSerie::new(time, HashMap::new());
        assert!(ts.is_empty());
    }

    #[test]
    fn timeserie_is_empty_with_no_vars() {
        let time = make_time_f32(5);
        let ts = TimeSerie::new(time, HashMap::new());
        assert!(ts.is_empty());
    }

    #[test]
    fn timeserie_default_is_empty() {
        let ts = TimeSerie::default();
        assert!(ts.is_empty());
        assert_eq!(ts.step_count(), 0);
        assert_eq!(ts.var_count(), 0);
    }

    #[test]
    fn timeserie_step_count_zero_for_empty_time() {
        let time = SlfArray1D::new(FloatSize::F32);
        let ts = TimeSerie::new(time, HashMap::new());
        assert_eq!(ts.step_count(), 0);
    }

    #[test]
    fn timeserie_var_count_zero_for_no_vars() {
        let time = make_time_f32(3);
        let ts = TimeSerie::new(time, HashMap::new());
        assert_eq!(ts.var_count(), 0);
    }

    // -----------------------------------------------------------------------
    // TimeSerie — assertion panics on mismatched step counts
    // -----------------------------------------------------------------------

    #[test]
    #[should_panic]
    fn timeserie_panics_when_var_has_too_few_steps() {
        let time = make_time_f32(5);
        let mut vars = HashMap::new();
        // Variable only has 3 snapshots but time has 5 steps
        vars.insert("DEPTH".to_string(), make_var_f32("DEPTH", "M", 3, 10, 0.0));
        TimeSerie::new(time, vars);
    }

    #[test]
    #[should_panic]
    fn timeserie_panics_when_var_has_too_many_steps() {
        let time = make_time_f32(3);
        let mut vars = HashMap::new();
        // Variable has 5 snapshots but time has only 3 steps
        vars.insert("DEPTH".to_string(), make_var_f32("DEPTH", "M", 5, 10, 0.0));
        TimeSerie::new(time, vars);
    }

    #[test]
    #[should_panic]
    fn timeserie_panics_when_one_of_several_vars_mismatches() {
        let time = make_time_f32(4);
        let mut vars = HashMap::new();
        vars.insert("DEPTH".to_string(), make_var_f32("DEPTH", "M", 4, 6, 0.0));
        vars.insert(
            "VELOCITY U".to_string(),
            make_var_f32("VELOCITY U", "M/S", 4, 6, 1.0),
        );
        // This one is wrong
        vars.insert(
            "VELOCITY V".to_string(),
            make_var_f32("VELOCITY V", "M/S", 2, 6, 2.0),
        );
        TimeSerie::new(time, vars);
    }
}
