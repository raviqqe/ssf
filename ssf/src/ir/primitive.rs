use crate::types;

#[derive(Clone, Debug, PartialEq)]
pub enum Primitive {
    Float64(f64),
    Integer8(u8),
    Integer64(u64),
}

impl Primitive {
    pub fn to_type(&self) -> types::Primitive {
        match self {
            Self::Float64(_) => types::Primitive::Float64,
            Self::Integer8(_) => types::Primitive::Integer8,
            Self::Integer64(_) => types::Primitive::Integer64,
        }
    }
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
