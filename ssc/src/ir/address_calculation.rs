use super::expression::Expression;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct AddressCalculation {
    pointer: Arc<Expression>,
    indices: Vec<Expression>,
}

impl AddressCalculation {
    pub fn new(pointer: impl Into<Expression>, indices: Vec<Expression>) -> Self {
        Self {
            pointer: pointer.into().into(),
            indices,
        }
    }

    pub fn pointer(&self) -> &Expression {
        &self.pointer
    }

    pub fn indices(&self) -> &[Expression] {
        &self.indices
    }
}
