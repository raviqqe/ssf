#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Primitive {
    Float64,
}

impl Primitive {
    pub fn to_id(&self) -> String {
        match self {
            Self::Float64 => "Float64".into(),
        }
    }
}
