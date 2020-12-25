#[derive(Clone, Debug, PartialEq)]
pub enum Primitive {
    Float32(f32),
    Float64(f64),
    Integer8(u8),
    Integer32(u32),
    Integer64(u64),
}

impl From<f32> for Primitive {
    fn from(number: f32) -> Self {
        Self::Float32(number)
    }
}

impl From<f64> for Primitive {
    fn from(number: f64) -> Self {
        Self::Float64(number)
    }
}

impl From<u32> for Primitive {
    fn from(number: u32) -> Self {
        Self::Integer32(number)
    }
}

impl From<u64> for Primitive {
    fn from(number: u64) -> Self {
        Self::Integer64(number)
    }
}
