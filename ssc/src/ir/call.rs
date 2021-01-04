use super::expression::Expression;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct Call {
    function: Arc<Expression>,
    arguments: Vec<Expression>,
    name: String,
}

impl Call {
    pub fn new(
        function: impl Into<Expression>,
        arguments: Vec<Expression>,
        name: impl Into<String>,
    ) -> Self {
        Self {
            function: function.into().into(),
            arguments,
            name: name.into(),
        }
    }

    pub fn function(&self) -> &Expression {
        &self.function
    }

    pub fn arguments(&self) -> &[Expression] {
        &self.arguments
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
