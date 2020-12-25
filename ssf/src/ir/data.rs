#[derive(Clone, Debug, PartialEq)]
pub struct Data {
    data: Vec<u8>,
}

impl Data {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }
}
