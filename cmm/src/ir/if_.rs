use super::expression::Expression;
use super::instruction::Instruction;

#[derive(Clone, Debug, PartialEq)]
pub struct If {
    condition: Expression,
    then: Vec<Instruction>,
    else_: Vec<Instruction>,
}

impl If {
    pub fn new(
        condition: impl Into<Expression>,
        then: Vec<Instruction>,
        else_: Vec<Instruction>,
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

    pub fn then(&self) -> &[Instruction] {
        &self.then
    }

    pub fn else_(&self) -> &[Instruction] {
        &self.else_
    }
}
