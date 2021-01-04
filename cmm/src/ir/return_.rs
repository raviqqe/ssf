use super::expression::Expression;

#[derive(Clone, Debug, PartialEq)]
pub struct Return {
    expression: Expression,
}

impl Return {
    pub fn new(expression: impl Into<Expression>) -> Self {
        Self {
            expression: expression.into(),
        }
    }

    pub fn expression(&self) -> &Expression {
        &self.expression
    }
}
