use super::alternative::Alternative;
use super::expression::Expression;
use super::instruction::Instruction;

#[derive(Clone, Debug, PartialEq)]
pub struct Switch {
    condition: Expression,
    alternatives: Vec<Alternative>,
    default_alternative: Vec<Instruction>,
}

impl Switch {
    pub fn new(
        condition: impl Into<Expression>,
        alternatives: Vec<Alternative>,
        default_alternative: Vec<Instruction>,
    ) -> Self {
        Self {
            condition: condition.into(),
            alternatives,
            default_alternative,
        }
    }

    pub fn condition(&self) -> &Expression {
        &self.condition
    }

    pub fn alternatives(&self) -> &[Alternative] {
        &self.alternatives
    }

    pub fn default_alternative(&self) -> &[Instruction] {
        &self.default_alternative
    }
}
