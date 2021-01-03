use super::type_::Type;
use std::sync::Arc;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Array {
    element: Arc<Type>,
    count: u64,
}

impl Array {
    pub fn new(element: impl Into<Type>, count: u64) -> Self {
        Self {
            element: element.into().into(),
            count,
        }
    }

    pub fn element(&self) -> &Type {
        &self.element
    }

    pub fn count(&self) -> u64 {
        self.count
    }
}
