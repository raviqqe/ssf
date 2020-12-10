use super::expression::Expression;
use crate::types::Type;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct ArrayIndexOperation {
    array: Arc<Expression>,
    index: Arc<Expression>,
}

impl ArrayIndexOperation {
    pub fn new(array: impl Into<Expression>, index: impl Into<Expression>) -> Self {
        Self {
            array: array.into().into(),
            index: index.into().into(),
        }
    }

    pub fn array(&self) -> &Expression {
        &self.array
    }

    pub fn index(&self) -> &Expression {
        &self.index
    }

    pub(crate) fn find_variables(&self) -> HashSet<String> {
        self.array
            .find_variables()
            .into_iter()
            .chain(self.index.find_variables())
            .collect()
    }

    pub(crate) fn infer_environment(&self, variables: &HashMap<String, Type>) -> Self {
        Self::new(
            self.array.infer_environment(variables),
            self.index.infer_environment(variables),
        )
    }

    pub(crate) fn convert_types(&self, convert: &impl Fn(&Type) -> Type) -> Self {
        Self::new(
            self.array.convert_types(convert),
            self.index.convert_types(convert),
        )
    }
}
