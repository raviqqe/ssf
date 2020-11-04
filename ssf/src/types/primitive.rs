#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Primitive {
    Float64,
    Integer8,
    Integer64,
}

impl Primitive {
    pub fn to_id(&self) -> String {
        format!("{:?}", self)
    }
}
