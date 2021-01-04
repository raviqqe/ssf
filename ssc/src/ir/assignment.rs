use super::expression::Expression;

#[derive(Clone, Debug, PartialEq)]
pub struct Assignment {
    expression: Expression,
    name: String,
}

impl Assignment {
    pub fn new(expression: impl Into<Expression>, name: impl Into<String>) -> Self {
        Self {
            expression: expression.into(),
            name: name.into(),
        }
    }

    pub fn expression(&self) -> &Expression {
        &self.expression
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
