#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Primitive {
    Float64,
    Integer64,
}

impl Primitive {
    pub fn to_id(&self) -> String {
        format!("{:?}", self)
    }
}
