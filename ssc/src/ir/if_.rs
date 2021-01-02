use super::expression::Expression;
use super::statement::Statement;

#[derive(Clone, Debug, PartialEq)]
pub struct If {
    condition: Expression,
    then: Vec<Statement>,
    else_: Vec<Statement>,
}

impl If {
    pub fn new(
        condition: impl Into<Expression>,
        then: Vec<Statement>,
        else_: Vec<Statement>,
    ) -> Self {
        Self {
            condition: condition.into(),
            then,
            else_,
        }
    }

    pub fn condition(&self) -> &Expression {
        &self.condition
    }

    pub fn then(&self) -> &[Statement] {
        &self.then
    }

    pub fn else_(&self) -> &[Statement] {
        &self.else_
    }
}
