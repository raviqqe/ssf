use super::type_::Type;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Constructor {
    elements: Vec<Type>,
}

impl Constructor {
    pub const fn new(elements: Vec<Type>) -> Self {
        Self { elements }
    }

    pub fn elements(&self) -> &[Type] {
        &self.elements
    }
}
