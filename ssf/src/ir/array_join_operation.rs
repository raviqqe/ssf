use super::expression::Expression;
use crate::types::Type;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

// TODO Replace this with ArraySetOperation.
#[derive(Clone, Debug, PartialEq)]
pub struct ArrayJoinOperation {
    lhs: Arc<Expression>,
    rhs: Arc<Expression>,
}

impl ArrayJoinOperation {
    pub fn new(lhs: impl Into<Expression>, rhs: impl Into<Expression>) -> Self {
        Self {
            lhs: lhs.into().into(),
            rhs: rhs.into().into(),
        }
    }

    pub fn lhs(&self) -> &Expression {
        &self.lhs
    }

    pub fn rhs(&self) -> &Expression {
        &self.rhs
    }

    pub(crate) fn find_variables(&self) -> HashSet<String> {
        self.lhs
            .find_variables()
            .into_iter()
            .chain(self.rhs.find_variables())
            .collect()
    }

    pub(crate) fn infer_environment(&self, variables: &HashMap<String, Type>) -> Self {
        Self::new(
            self.lhs.infer_environment(variables),
            self.rhs.infer_environment(variables),
        )
    }

    pub(crate) fn convert_types(&self, convert: &impl Fn(&Type) -> Type) -> Self {
        Self::new(
            self.lhs.convert_types(convert),
            self.rhs.convert_types(convert),
        )
    }
}
