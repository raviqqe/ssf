use super::expression::Expression;

#[derive(Clone, Debug, PartialEq)]
pub struct Assignment {
    name: String,
    expression: Expression,
}

impl Assignment {
    pub fn new(name: impl Into<String>, expression: impl Into<Expression>) -> Self {
        Self {
            name: name.into(),
            expression: expression.into(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn expression(&self) -> &Expression {
        &self.expression
    }
}
