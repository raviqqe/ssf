use crate::utilities;

#[derive(Clone, Debug, PartialEq)]
pub struct VariableBuilder {
    typed_expression: fmm::build::TypedExpression,
    type_: fmm::types::Type,
}

impl VariableBuilder {
    pub fn new(typed_expression: fmm::build::TypedExpression) -> Self {
        Self::with_type(typed_expression.clone(), typed_expression.type_().clone())
    }

    pub fn with_type(
        typed_expression: fmm::build::TypedExpression,
        type_: impl Into<fmm::types::Type>,
    ) -> Self {
        Self {
            typed_expression,
            type_: type_.into(),
        }
    }

    pub fn build(&self, builder: &fmm::build::InstructionBuilder) -> fmm::build::TypedExpression {
        utilities::bitcast(&builder, self.typed_expression.clone(), self.type_.clone())
    }
}

impl From<fmm::build::TypedExpression> for VariableBuilder {
    fn from(typed_expression: fmm::build::TypedExpression) -> Self {
        Self::new(typed_expression)
    }
}
