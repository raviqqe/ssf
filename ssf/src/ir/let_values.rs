use super::expression::Expression;
use super::definition::Definition;
use crate::types::Type;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq)]
pub struct LetValues {
    definitions: Vec<Definition>,
    expression: Box<Expression>,
}

impl LetValues {
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
        let mut definitions = Vec::with_capacity(self.definitions.len());

        for definition in &self.definitions {
            definitions.push(definition.rename_variables(&names));
            names.remove(definition.name());
        }

        Self::new(definitions, self.expression.rename_variables(&names))
    }

    pub(crate) fn find_variables(&self, excluded_variables: &HashSet<String>) -> HashSet<String> {
        let mut excluded_variables = excluded_variables.clone();
        let mut variables = HashSet::new();

        for definition in &self.definitions {
            variables.extend(definition.find_variables(&excluded_variables));
            excluded_variables.insert(definition.name().into());
        }

        variables.extend(self.expression.find_variables(&excluded_variables));

        variables
    }

    pub(crate) fn infer_environment(
        &self,
        variables: &HashMap<String, Type>,
        global_variables: &HashSet<String>,
    ) -> Self {
        let mut variables = variables.clone();
        let mut definitions = vec![];

        for definition in &self.definitions {
            definitions.push(definition.infer_environment(&variables, global_variables));
            variables.insert(
                definition.name().into(),
                definition.type_().clone().into(),
            );
        }

        Self::new(
            definitions,
            self.expression
                .infer_environment(&variables, global_variables),
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
