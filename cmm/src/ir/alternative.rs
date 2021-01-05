use super::instruction::Instruction;
use super::primitive::Primitive;

#[derive(Clone, Debug, PartialEq)]
pub struct Alternative {
    condition: Primitive,
    instructions: Vec<Instruction>,
}

impl Alternative {
    pub fn new(condition: impl Into<Primitive>, instructions: Vec<Instruction>) -> Self {
        Self {
            condition: condition.into(),
            instructions,
        }
    }

    pub fn condition(&self) -> Primitive {
        self.condition
    }

    pub fn instructions(&self) -> &[Instruction] {
        &self.instructions
    }
}
