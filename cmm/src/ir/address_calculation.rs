use super::expression::Expression;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct AddressCalculation {
    pointer: Arc<Expression>,
    indices: Vec<Expression>,
    name: String,
}

impl AddressCalculation {
    pub fn new(
        pointer: impl Into<Expression>,
        indices: Vec<Expression>,
        name: impl Into<String>,
    ) -> Self {
        Self {
            pointer: pointer.into().into(),
            indices,
            name: name.into(),
        }
    }

    pub fn pointer(&self) -> &Expression {
        &self.pointer
    }

    pub fn indices(&self) -> &[Expression] {
        &self.indices
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
