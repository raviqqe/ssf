use super::expression::Expression;
use crate::types::Type;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct ArrayMultipleElement {
    array: Arc<Expression>,
    size: Arc<Expression>,
}

impl ArrayMultipleElement {
    pub fn new(array: impl Into<Expression>, size: impl Into<Expression>) -> Self {
        Self {
            array: array.into().into(),
            size: size.into().into(),
        }
    }

    pub fn array(&self) -> &Expression {
        &self.array
    }

    pub fn size(&self) -> &Expression {
        &self.size
    }

    pub(crate) fn find_variables(&self) -> HashSet<String> {
        self.array
            .find_variables()
            .into_iter()
            .chain(self.size.find_variables())
            .collect()
    }

    pub(crate) fn infer_environment(&self, variables: &HashMap<String, Type>) -> Self {
        Self::new(
            self.array.infer_environment(variables),
            self.size.infer_environment(variables),
        )
    }

    pub(crate) fn convert_types(&self, convert: &impl Fn(&Type) -> Type) -> Self {
        Self::new(
            self.array.convert_types(convert),
            self.size.convert_types(convert),
        )
    }
}
