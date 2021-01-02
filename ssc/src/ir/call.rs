use super::expression::Expression;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct Call {
    function: Arc<Expression>,
    arguments: Vec<Expression>,
}

impl Call {
    pub fn new(function: impl Into<Expression>, arguments: Vec<Expression>) -> Self {
        Self {
            function: function.into().into(),
            arguments,
        }
    }

    pub fn function(&self) -> &Expression {
        &self.function
    }

    pub fn arguments(&self) -> &[Expression] {
        &self.arguments
    }
}
