use super::expression::Expression;
use super::statement::Statement;

#[derive(Clone, Debug, PartialEq)]
pub struct Alternative {
    condition: Expression,
    statements: Vec<Statement>,
}

impl Alternative {
    pub fn new(condition: impl Into<Expression>, statements: Vec<Statement>) -> Self {
        Self {
            condition: condition.into(),
            statements,
        }
    }

    pub fn condition(&self) -> &Expression {
        &self.condition
    }

    pub fn statements(&self) -> &[Statement] {
        &self.statements
    }
}
