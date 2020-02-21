#[derive(Clone, Debug, PartialEq)]
pub enum Primitive {
    Float64(f64),
}

impl From<f64> for Primitive {
    fn from(number: f64) -> Self {
        Self::Float64(number)
    }
}
