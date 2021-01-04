use crate::types::Type;

#[derive(Clone, Debug, PartialEq)]
pub struct AllocateStack {
    type_: Type,
    name: String,
}

impl AllocateStack {
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
