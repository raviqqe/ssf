use super::expression::Expression;
use super::instruction::Instruction;

#[derive(Clone, Debug, PartialEq)]
pub struct Alternative {
    condition: Expression,
    instructions: Vec<Instruction>,
}

impl Alternative {
    pub fn new(condition: impl Into<Expression>, instructions: Vec<Instruction>) -> Self {
        Self {
            condition: condition.into(),
            instructions,
        }
    }

    pub fn condition(&self) -> &Expression {
        &self.condition
    }

    pub fn instructions(&self) -> &[Instruction] {
        &self.instructions
    }
}
