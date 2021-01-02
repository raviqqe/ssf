use super::expression::Expression;
use crate::types::Type;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct Bitcast {
    expression: Arc<Expression>,
    type_: Type,
}

impl Bitcast {
    pub fn new(expression: impl Into<Expression>, type_: impl Into<Type>) -> Self {
        Self {
            expression: expression.into().into(),
            type_: type_.into(),
        }
    }

    pub fn expression(&self) -> &Expression {
        &self.expression
    }

    pub fn type_(&self) -> &Type {
        &self.type_
    }
}
