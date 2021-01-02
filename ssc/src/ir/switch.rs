use super::alternative::Alternative;
use super::expression::Expression;
use super::statement::Statement;

#[derive(Clone, Debug, PartialEq)]
pub struct Switch {
    condition: Expression,
    alternatives: Vec<Alternative>,
    default_alternative: Vec<Statement>,
}

impl Switch {
    pub fn new(
        condition: impl Into<Expression>,
        alternatives: Vec<Alternative>,
        default_alternative: Vec<Statement>,
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

    pub fn default_alternative(&self) -> &[Statement] {
        &self.default_alternative
    }
}
