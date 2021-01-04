use super::expression::Expression;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct CompareAndSwap {
    pointer: Arc<Expression>,
    old_value: Arc<Expression>,
    new_value: Arc<Expression>,
    name: String,
}

impl CompareAndSwap {
    pub fn new(
        pointer: impl Into<Expression>,
        old_value: impl Into<Expression>,
        new_value: impl Into<Expression>,
        name: impl Into<String>,
    ) -> Self {
        Self {
            pointer: pointer.into().into(),
            old_value: old_value.into().into(),
            new_value: new_value.into().into(),
            name: name.into(),
        }
    }

    pub fn pointer(&self) -> &Expression {
        &self.pointer
    }

    pub fn old_value(&self) -> &Expression {
        &self.old_value
    }

    pub fn new_value(&self) -> &Expression {
        &self.new_value
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
