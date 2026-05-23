//! # Size-independent containers
//!
//! This means vectors 1D or 2D that can contains either f32 or f64

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FloatSize {
    F32,
    F64,
}

#[derive(Debug)]
pub enum SlfArray1D {
    Float(Vec<f32>),
    Double(Vec<f64>),
}

impl SlfArray1D {
    pub fn new(float_size: FloatSize) -> Self {
        match float_size {
            FloatSize::F32 => SlfArray1D::Float(Vec::new()),
            FloatSize::F64 => SlfArray1D::Double(Vec::new()),
        }
    }

    pub fn with_capacity(float_size: FloatSize, capacity: usize) -> Self {
        match float_size {
            FloatSize::F32 => SlfArray1D::Float(Vec::with_capacity(capacity)),
            FloatSize::F64 => SlfArray1D::Double(Vec::with_capacity(capacity)),
        }
    }

    pub fn reserve(&mut self, additional: usize) {
        match self {
            SlfArray1D::Float(v) => v.reserve(additional),
            SlfArray1D::Double(v) => v.reserve(additional),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            SlfArray1D::Float(v) => v.len(),
            SlfArray1D::Double(v) => v.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            SlfArray1D::Float(v) => v.is_empty(),
            SlfArray1D::Double(v) => v.is_empty(),
        }
    }
}

impl Default for SlfArray1D {
    fn default() -> Self {
        SlfArray1D::Float(Vec::new())
    }
}

impl From<SlfArray1D> for Vec<f32> {
    fn from(value: SlfArray1D) -> Self {
        match value {
            SlfArray1D::Float(v) => v,
            SlfArray1D::Double(v) => v.into_iter().map(|x| x as f32).collect(),
        }
    }
}

impl From<SlfArray1D> for Vec<f64> {
    fn from(value: SlfArray1D) -> Self {
        match value {
            SlfArray1D::Float(v) => v.into_iter().map(|x| x as f64).collect(),
            SlfArray1D::Double(v) => v,
        }
    }
}

impl From<Vec<f32>> for SlfArray1D {
    fn from(value: Vec<f32>) -> Self {
        SlfArray1D::Float(value)
    }
}

impl From<Vec<f64>> for SlfArray1D {
    fn from(value: Vec<f64>) -> Self {
        SlfArray1D::Double(value)
    }
}

#[derive(Debug)]
pub enum SlfArray2D {
    Float { x: Vec<f32>, y: Vec<f32> },
    Double { x: Vec<f64>, y: Vec<f64> },
}

impl SlfArray2D {
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        match self {
            SlfArray2D::Float { x, y } => {
                debug_assert_eq!(x.len(), y.len());
                x.len()
            }
            SlfArray2D::Double { x, y } => {
                debug_assert_eq!(x.len(), y.len());
                x.len()
            }
        }
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        match self {
            SlfArray2D::Float { x, y } => {
                debug_assert_eq!(x.is_empty(), y.is_empty());
                x.is_empty()
            }
            SlfArray2D::Double { x, y } => {
                debug_assert_eq!(x.is_empty(), y.is_empty());
                x.is_empty()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn float(x: Vec<f32>, y: Vec<f32>) -> SlfArray2D {
        SlfArray2D::Float { x, y }
    }

    fn double(x: Vec<f64>, y: Vec<f64>) -> SlfArray2D {
        SlfArray2D::Double { x, y }
    }

    // -----------------------------------------------------------------------
    // Constructors
    // -----------------------------------------------------------------------

    #[test]
    fn new_f32_produces_float_variant() {
        let a = SlfArray1D::new(FloatSize::F32);
        assert!(matches!(a, SlfArray1D::Float(_)));
    }

    #[test]
    fn new_f64_produces_double_variant() {
        let a = SlfArray1D::new(FloatSize::F64);
        assert!(matches!(a, SlfArray1D::Double(_)));
    }

    #[test]
    fn new_is_empty() {
        assert!(SlfArray1D::new(FloatSize::F32).is_empty());
        assert!(SlfArray1D::new(FloatSize::F64).is_empty());
    }

    #[test]
    fn new_has_zero_len() {
        assert_eq!(SlfArray1D::new(FloatSize::F32).len(), 0);
        assert_eq!(SlfArray1D::new(FloatSize::F64).len(), 0);
    }

    #[test]
    fn with_capacity_f32_is_empty() {
        let a = SlfArray1D::with_capacity(FloatSize::F32, 64);
        assert!(matches!(a, SlfArray1D::Float(_)));
        assert!(a.is_empty()); // capacity ≠ length
    }

    #[test]
    fn with_capacity_f64_is_empty() {
        let a = SlfArray1D::with_capacity(FloatSize::F64, 64);
        assert!(matches!(a, SlfArray1D::Double(_)));
        assert!(a.is_empty());
    }

    #[test]
    fn default_is_float_empty() {
        let a = SlfArray1D::default();
        assert!(matches!(a, SlfArray1D::Float(_)));
        assert!(a.is_empty());
    }

    // -----------------------------------------------------------------------
    // len / is_empty
    // -----------------------------------------------------------------------

    #[test]
    fn len_reflects_element_count_f32() {
        let a = SlfArray1D::Float(vec![1.0, 2.0, 3.0]);
        assert_eq!(a.len(), 3);
    }

    #[test]
    fn len_reflects_element_count_f64() {
        let a = SlfArray1D::Double(vec![1.0, 2.0]);
        assert_eq!(a.len(), 2);
    }

    #[test]
    fn is_empty_false_when_data_present_f32() {
        let a = SlfArray1D::Float(vec![0.0]);
        assert!(!a.is_empty());
    }

    #[test]
    fn is_empty_false_when_data_present_f64() {
        let a = SlfArray1D::Double(vec![0.0]);
        assert!(!a.is_empty());
    }

    // -----------------------------------------------------------------------
    // reserve
    // -----------------------------------------------------------------------

    #[test]
    fn reserve_does_not_change_len_f32() {
        let mut a = SlfArray1D::new(FloatSize::F32);
        a.reserve(128);
        assert_eq!(a.len(), 0);
    }

    #[test]
    fn reserve_does_not_change_len_f64() {
        let mut a = SlfArray1D::new(FloatSize::F64);
        a.reserve(128);
        assert_eq!(a.len(), 0);
    }

    // -----------------------------------------------------------------------
    // From<Vec<f32>> / From<Vec<f64>>
    // -----------------------------------------------------------------------

    #[test]
    fn from_vec_f32_produces_float_variant() {
        let a: SlfArray1D = vec![1.0f32, 2.0].into();
        assert!(matches!(a, SlfArray1D::Float(_)));
    }

    #[test]
    fn from_vec_f64_produces_double_variant() {
        let a: SlfArray1D = vec![1.0f64, 2.0].into();
        assert!(matches!(a, SlfArray1D::Double(_)));
    }

    #[test]
    fn from_empty_vec_f32_is_empty() {
        let a: SlfArray1D = Vec::<f32>::new().into();
        assert!(a.is_empty());
    }

    #[test]
    fn from_empty_vec_f64_is_empty() {
        let a: SlfArray1D = Vec::<f64>::new().into();
        assert!(a.is_empty());
    }

    // -----------------------------------------------------------------------
    // Into<Vec<f32>> / Into<Vec<f64>> - identity conversions
    // -----------------------------------------------------------------------

    #[test]
    fn float_variant_round_trips_to_vec_f32() {
        let src = vec![1.0f32, 2.0, 3.0];
        let a: SlfArray1D = src.clone().into();
        let out: Vec<f32> = a.into();
        assert_eq!(out, src);
    }

    #[test]
    fn double_variant_round_trips_to_vec_f64() {
        let src = vec![1.0f64, 2.0, 3.0];
        let a: SlfArray1D = src.clone().into();
        let out: Vec<f64> = a.into();
        assert_eq!(out, src);
    }

    #[test]
    fn empty_float_variant_converts_to_empty_vec_f32() {
        let a = SlfArray1D::new(FloatSize::F32);
        let out: Vec<f32> = a.into();
        assert!(out.is_empty());
    }

    #[test]
    fn empty_double_variant_converts_to_empty_vec_f64() {
        let a = SlfArray1D::new(FloatSize::F64);
        let out: Vec<f64> = a.into();
        assert!(out.is_empty());
    }

    // -----------------------------------------------------------------------
    // Into<Vec<f32>> / Into<Vec<f64>> - cross-type conversions
    // -----------------------------------------------------------------------

    #[test]
    fn float_variant_converts_to_vec_f64() {
        let a: SlfArray1D = vec![1.0f32, 2.0, 3.0].into();
        let out: Vec<f64> = a.into();
        assert_eq!(out.len(), 3);
        assert!((out[0] - 1.0f64).abs() < 1e-6);
        assert!((out[1] - 2.0f64).abs() < 1e-6);
        assert!((out[2] - 3.0f64).abs() < 1e-6);
    }

    #[test]
    fn double_variant_converts_to_vec_f32() {
        let a: SlfArray1D = vec![1.0f64, 2.0, 3.0].into();
        let out: Vec<f32> = a.into();
        assert_eq!(out.len(), 3);
        assert!((out[0] - 1.0f32).abs() < 1e-6);
        assert!((out[1] - 2.0f32).abs() < 1e-6);
        assert!((out[2] - 3.0f32).abs() < 1e-6);
    }

    #[test]
    fn empty_float_variant_converts_to_empty_vec_f64() {
        let a = SlfArray1D::new(FloatSize::F32);
        let out: Vec<f64> = a.into();
        assert!(out.is_empty());
    }

    #[test]
    fn empty_double_variant_converts_to_empty_vec_f32() {
        let a = SlfArray1D::new(FloatSize::F64);
        let out: Vec<f32> = a.into();
        assert!(out.is_empty());
    }

    #[test]
    fn float_to_f64_preserves_special_values() {
        let a: SlfArray1D = vec![f32::INFINITY, f32::NEG_INFINITY, f32::NAN].into();
        let out: Vec<f64> = a.into();
        assert!(out[0].is_infinite() && out[0].is_sign_positive());
        assert!(out[1].is_infinite() && out[1].is_sign_negative());
        assert!(out[2].is_nan());
    }

    #[test]
    fn double_to_f32_preserves_special_values() {
        let a: SlfArray1D = vec![f64::INFINITY, f64::NEG_INFINITY, f64::NAN].into();
        let out: Vec<f32> = a.into();
        assert!(out[0].is_infinite() && out[0].is_sign_positive());
        assert!(out[1].is_infinite() && out[1].is_sign_negative());
        assert!(out[2].is_nan());
    }

    #[test]
    fn double_to_f32_large_value_saturates_to_infinity() {
        // f64 values beyond f32::MAX become f32::INFINITY on cast
        let a: SlfArray1D = vec![f64::MAX].into();
        let out: Vec<f32> = a.into();
        assert!(out[0].is_infinite());
    }

    // -----------------------------------------------------------------------
    // SlfArray2D - Variant identity
    // -----------------------------------------------------------------------

    #[test]
    fn float_variant_is_float() {
        assert!(matches!(float(vec![], vec![]), SlfArray2D::Float { .. }));
    }

    #[test]
    fn double_variant_is_double() {
        assert!(matches!(double(vec![], vec![]), SlfArray2D::Double { .. }));
    }

    // -----------------------------------------------------------------------
    // len - Float
    // -----------------------------------------------------------------------

    #[test]
    fn float_len_zero_when_empty() {
        assert_eq!(float(vec![], vec![]).len(), 0);
    }

    #[test]
    fn float_len_reflects_point_count() {
        assert_eq!(float(vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]).len(), 3);
    }

    #[test]
    fn float_len_single_point() {
        assert_eq!(float(vec![0.0], vec![0.0]).len(), 1);
    }

    #[test]
    #[should_panic]
    fn float_len_panics_when_x_longer_than_y() {
        float(vec![1.0, 2.0], vec![1.0]).len();
    }

    #[test]
    #[should_panic]
    fn float_len_panics_when_y_longer_than_x() {
        float(vec![1.0], vec![1.0, 2.0]).len();
    }

    // -----------------------------------------------------------------------
    // len - Double
    // -----------------------------------------------------------------------

    #[test]
    fn double_len_zero_when_empty() {
        assert_eq!(double(vec![], vec![]).len(), 0);
    }

    #[test]
    fn double_len_reflects_point_count() {
        assert_eq!(double(vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]).len(), 3);
    }

    #[test]
    fn double_len_single_point() {
        assert_eq!(double(vec![0.0], vec![0.0]).len(), 1);
    }

    #[test]
    #[should_panic]
    fn double_len_panics_when_x_longer_than_y() {
        double(vec![1.0, 2.0], vec![1.0]).len();
    }

    #[test]
    #[should_panic]
    fn double_len_panics_when_y_longer_than_x() {
        double(vec![1.0], vec![1.0, 2.0]).len();
    }

    // -----------------------------------------------------------------------
    // is_empty - Float
    // -----------------------------------------------------------------------

    #[test]
    fn float_is_empty_when_both_empty() {
        assert!(float(vec![], vec![]).is_empty());
    }

    #[test]
    fn float_is_not_empty_when_data_present() {
        assert!(!float(vec![1.0], vec![2.0]).is_empty());
    }

    #[test]
    #[should_panic]
    fn float_is_empty_panics_when_only_x_is_empty() {
        float(vec![], vec![1.0]).is_empty();
    }

    #[test]
    #[should_panic]
    fn float_is_empty_panics_when_only_y_is_empty() {
        float(vec![1.0], vec![]).is_empty();
    }

    // -----------------------------------------------------------------------
    // is_empty - Double
    // -----------------------------------------------------------------------

    #[test]
    fn double_is_empty_when_both_empty() {
        assert!(double(vec![], vec![]).is_empty());
    }

    #[test]
    fn double_is_not_empty_when_data_present() {
        assert!(!double(vec![1.0], vec![2.0]).is_empty());
    }

    #[test]
    #[should_panic]
    fn double_is_empty_panics_when_only_x_is_empty() {
        double(vec![], vec![1.0]).is_empty();
    }

    #[test]
    #[should_panic]
    fn double_is_empty_panics_when_only_y_is_empty() {
        double(vec![1.0], vec![]).is_empty();
    }

    // -----------------------------------------------------------------------
    // Data integrity - coordinates are stored and retrievable
    // -----------------------------------------------------------------------

    #[test]
    fn float_stores_coordinates() {
        let a = float(vec![1.0, 2.0], vec![3.0, 4.0]);
        if let SlfArray2D::Float { x, y } = a {
            assert_eq!(x, vec![1.0f32, 2.0]);
            assert_eq!(y, vec![3.0f32, 4.0]);
        } else {
            panic!("Expected Float variant");
        }
    }

    #[test]
    fn double_stores_coordinates() {
        let a = double(vec![1.0, 2.0], vec![3.0, 4.0]);
        if let SlfArray2D::Double { x, y } = a {
            assert_eq!(x, vec![1.0f64, 2.0]);
            assert_eq!(y, vec![3.0f64, 4.0]);
        } else {
            panic!("Expected Double variant");
        }
    }
}
