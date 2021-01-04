use super::type_::Type;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Record {
    elements: Vec<Type>,
}

impl Record {
    pub const fn new(elements: Vec<Type>) -> Self {
        Self { elements }
    }

    pub fn elements(&self) -> &[Type] {
        &self.elements
    }
}
