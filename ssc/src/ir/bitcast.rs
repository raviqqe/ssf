use super::expression::Expression;
use crate::types::Type;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct Bitcast {
    expression: Arc<Expression>,
    type_: Type,
    name: String,
}

impl Bitcast {
    pub fn new(
        expression: impl Into<Expression>,
        type_: impl Into<Type>,
        name: impl Into<String>,
    ) -> Self {
        Self {
            expression: expression.into().into(),
            type_: type_.into(),
            name: name.into(),
        }
    }

    pub fn expression(&self) -> &Expression {
        &self.expression
    }

    pub fn type_(&self) -> &Type {
        &self.type_
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
