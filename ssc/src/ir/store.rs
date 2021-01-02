use super::expression::Expression;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct Store {
    value: Arc<Expression>,
    pointer: Arc<Expression>,
}

impl Store {
    pub fn new(value: impl Into<Expression>, pointer: impl Into<Expression>) -> Self {
        Self {
            value: value.into().into(),
            pointer: pointer.into().into(),
        }
    }

    pub fn value(&self) -> &Expression {
        &self.value
    }

    pub fn pointer(&self) -> &Expression {
        &self.pointer
    }
}
