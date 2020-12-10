use super::array_multiple_element::ArrayMultipleElement;
use super::expression::Expression;
use crate::types::Type;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq)]
pub enum ArrayElement {
    Single(Expression),
    Multiple(ArrayMultipleElement),
}

impl ArrayElement {
    pub(crate) fn find_variables(&self) -> HashSet<String> {
        match self {
            Self::Multiple(element) => element.find_variables(),
            Self::Single(element) => element.find_variables(),
        }
    }

    pub(crate) fn infer_environment(&self, variables: &HashMap<String, Type>) -> Self {
        match self {
            Self::Multiple(element) => element.infer_environment(variables).into(),
            Self::Single(element) => element.infer_environment(variables).into(),
        }
    }

    pub(crate) fn convert_types(&self, convert: &impl Fn(&Type) -> Type) -> Self {
        match self {
            Self::Multiple(element) => element.convert_types(convert).into(),
            Self::Single(element) => element.convert_types(convert).into(),
        }
    }
}

impl<T: Into<Expression>> From<T> for ArrayElement {
    fn from(element: T) -> Self {
        Self::Single(element.into())
    }
}

impl From<ArrayMultipleElement> for ArrayElement {
    fn from(element: ArrayMultipleElement) -> Self {
        Self::Multiple(element)
    }
}
