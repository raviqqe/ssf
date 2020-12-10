use super::array_element::ArrayElement;
use crate::types::Type;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq)]
pub struct Array {
    element_type: Type,
    elements: Vec<ArrayElement>,
}

impl Array {
    pub fn new(element_type: impl Into<Type>, elements: Vec<ArrayElement>) -> Self {
        Self {
            element_type: element_type.into(),
            elements,
        }
    }

    pub fn element_type(&self) -> &Type {
        &self.element_type
    }

    pub fn elements(&self) -> &[ArrayElement] {
        &self.elements
    }

    pub(crate) fn find_variables(&self) -> HashSet<String> {
        self.elements
            .iter()
            .map(|element| element.find_variables())
            .flatten()
            .collect()
    }

    pub(crate) fn infer_environment(&self, variables: &HashMap<String, Type>) -> Self {
        Self::new(
            self.element_type.clone(),
            self.elements
                .iter()
                .map(|element| element.infer_environment(variables))
                .collect(),
        )
    }

    pub(crate) fn convert_types(&self, convert: &impl Fn(&Type) -> Type) -> Self {
        Self::new(
            convert(&self.element_type),
            self.elements
                .iter()
                .map(|element| element.convert_types(convert))
                .collect(),
        )
    }
}
