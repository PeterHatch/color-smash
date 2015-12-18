use std::cmp::Ordering;

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct n64(f64);

impl Eq for n64 {}

impl From<f64> for n64 {
    fn from(val: f64) -> n64 {
        debug_assert!(!val.is_nan(), "Tried to create n64 that was NaN");
        n64(val)
    }
}

impl From<n64> for f64 {
    fn from(val: n64) -> f64 {
        val.0
    }
}

impl Ord for n64 {
    fn cmp(&self, other: &n64) -> Ordering {
        self.partial_cmp(other).expect("NaN found in n64 comparison")
    }
}
