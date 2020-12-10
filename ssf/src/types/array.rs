use super::type_::Type;
use std::sync::Arc;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Array {
    element: Arc<Type>,
}

impl Array {
    pub fn new(element: impl Into<Type>) -> Self {
        Self {
            element: element.into().into(),
        }
    }

    pub fn element(&self) -> &Type {
        &self.element
    }

    pub fn to_id(&self) -> String {
        format!("[{}]", self.element.to_id())
    }
}
