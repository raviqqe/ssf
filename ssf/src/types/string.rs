#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct String {
    value: Vec<u8>,
}

impl String {
    pub const fn new(value: Vec<u8>) -> Self {
        Self { value }
    }

    pub fn value(&self) -> &[u8] {
        &self.value
    }

    pub fn to_id(&self) -> std::string::String {
        "string".into()
    }
}
