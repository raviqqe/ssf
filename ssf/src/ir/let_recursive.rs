use super::definition::Definition;
use super::expression::Expression;
use crate::types::Type;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq)]
pub struct LetRecursive {
    definitions: Vec<Definition>,
    expression: Box<Expression>,
}

impl LetRecursive {
    pub fn new(definitions: Vec<Definition>, expression: impl Into<Expression>) -> Self {
        Self {
            definitions,
            expression: Box::new(expression.into()),
        }
    }

    pub fn definitions(&self) -> &[Definition] {
        &self.definitions
    }

    pub fn expression(&self) -> &Expression {
        &self.expression
    }

    pub(crate) fn rename_variables(&self, names: &HashMap<String, String>) -> Self {
        let mut names = names.clone();

        for definition in &self.definitions {
            names.remove(definition.name());
        }

        Self::new(
            self.definitions
                .iter()
                .map(|definition| definition.rename_variables(&names))
                .collect(),
            self.expression.rename_variables(&names),
        )
    }

    pub(crate) fn find_free_variables(&self, initialized: bool) -> HashSet<String> {
        let mut variables = HashSet::new();

        for definition in &self.definitions {
            variables.extend(definition.find_free_variables(initialized));
        }

        variables.extend(self.expression.find_free_variables(initialized));

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

        let mut definitions = vec![];

        for definition in &self.definitions {
            definitions.push(definition.infer_environment(&variables));
        }

        Self::new(definitions, self.expression.infer_environment(&variables))
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
