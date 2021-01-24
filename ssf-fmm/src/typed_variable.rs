use crate::utilities;

#[derive(Clone, Debug, PartialEq)]
pub struct TypedVariable {
    typed_expression: fmm::build::TypedExpression,
    type_: fmm::types::Type,
}

impl TypedVariable {
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

    pub fn build(&self, builder: &fmm::build::BlockBuilder) -> fmm::build::TypedExpression {
        utilities::bitcast(&builder, self.typed_expression.clone(), self.type_.clone())
    }

    pub fn type_(&self) -> &fmm::types::Type {
        &self.type_
    }
}

impl From<fmm::build::TypedExpression> for TypedVariable {
    fn from(typed_expression: fmm::build::TypedExpression) -> Self {
        Self::new(typed_expression)
    }
}
