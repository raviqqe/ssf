use crate::types::Type;

#[derive(Clone, Debug, PartialEq)]
pub struct AllocateHeap {
    type_: Type,
    name: String,
}

impl AllocateHeap {
    pub fn new(type_: impl Into<Type>, name: impl Into<String>) -> Self {
        Self {
            type_: type_.into(),
            name: name.into(),
        }
    }

    pub fn type_(&self) -> &Type {
        &self.type_
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
