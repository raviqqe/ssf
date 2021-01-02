use super::type_::Type;
use std::sync::Arc;

// This is the first-class type differently from C.
// How it's compiled depends on backend.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Function {
    arguments: Vec<Type>,
    result: Arc<Type>,
}

impl Function {
    pub fn new(arguments: Vec<Type>, result: impl Into<Type>) -> Self {
        Self {
            arguments,
            result: result.into().into(),
        }
    }

    pub fn arguments(&self) -> &[Type] {
        &self.arguments
    }

    pub fn result(&self) -> &Type {
        &self.result
    }
}
