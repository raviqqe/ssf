use super::expression::Expression;
use super::primitive::Primitive;
use crate::types::Type;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq)]
pub struct PrimitiveAlternative {
    primitive: Primitive,
    expression: Expression,
}

impl PrimitiveAlternative {
    pub fn new(primitive: impl Into<Primitive>, expression: impl Into<Expression>) -> Self {
        Self {
            primitive: primitive.into(),
            expression: expression.into(),
        }
    }

    pub fn primitive(&self) -> &Primitive {
        &self.primitive
    }

    pub fn expression(&self) -> &Expression {
        &self.expression
    }

    pub(crate) fn find_variables(&self) -> HashSet<String> {
        self.expression.find_variables()
    }

    pub(crate) fn infer_environment(&self, variables: &HashMap<String, Type>) -> Self {
        Self {
            primitive: self.primitive.clone(),
            expression: self.expression.infer_environment(variables),
        }
    }

    pub(crate) fn convert_types(&self, convert: &impl Fn(&Type) -> Type) -> Self {
        Self {
            primitive: self.primitive.clone(),
            expression: self.expression.convert_types(convert),
        }
    }
}
