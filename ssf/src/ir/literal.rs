#[derive(Clone, Debug, PartialEq)]
pub enum Literal {
    Float64(f64),
}

impl From<f64> for Literal {
    fn from(number: f64) -> Self {
        Self::Float64(number)
    }
}
