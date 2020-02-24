#[derive(Clone, Debug, PartialEq)]
pub enum Primitive {
    Float64(f64),
    Integer64(u64),
}

impl From<f64> for Primitive {
    fn from(number: f64) -> Self {
        Self::Float64(number)
    }
}

impl From<u64> for Primitive {
    fn from(number: u64) -> Self {
        Self::Integer64(number)
    }
}
