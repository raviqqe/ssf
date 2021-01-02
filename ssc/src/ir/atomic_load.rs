use super::expression::Expression;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct AtomicLoad {
    pointer: Arc<Expression>,
}

impl AtomicLoad {
    pub fn new(pointer: impl Into<Expression>) -> Self {
        Self {
            pointer: pointer.into().into(),
        }
    }

    pub fn pointer(&self) -> &Expression {
        &self.pointer
    }
}
