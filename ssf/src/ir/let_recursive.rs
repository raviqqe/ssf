use super::definition::Definition;
use super::expression::Expression;
use crate::types::Type;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct LetRecursive {
    definitions: Vec<Definition>,
    expression: Arc<Expression>,
}

impl LetRecursive {
    pub fn new(definitions: Vec<Definition>, expression: impl Into<Expression>) -> Self {
        Self {
            definitions,
            expression: Arc::new(expression.into()),
        }
    }

    pub fn definitions(&self) -> &[Definition] {
        &self.definitions
    }

    pub fn expression(&self) -> &Expression {
        &self.expression
    }

    pub(crate) fn find_variables(&self) -> HashSet<String> {
        let mut variables = self.expression.find_variables();

        for definition in &self.definitions {
            variables.extend(definition.find_variables());
        }

        for definition in &self.definitions {
            variables.remove(definition.name());
        }

        variables
    }

    pub(crate) fn infer_environment(&self, variables: &HashMap<String, Type>) -> Self {
        let mut variables = variables.clone();

        for definition in &self.definitions {
            variables.insert(definition.name().into(), definition.type_().clone().into());
        }

        Self::new(
            self.definitions
                .iter()
                .map(|definition| definition.infer_environment(&variables))
                .collect(),
            self.expression.infer_environment(&variables),
        )
    }

    pub(crate) fn convert_types(&self, convert: &impl Fn(&Type) -> Type) -> Self {
        Self::new(
            self.definitions
                .iter()
                .map(|definition| definition.convert_types(convert))
                .collect(),
            self.expression.convert_types(convert),
        )
    }
}
