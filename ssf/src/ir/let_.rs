use super::definition::Definition;
use super::expression::Expression;
use crate::types::Type;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq)]
pub struct Let {
    definitions: Vec<Definition>,
    expression: Box<Expression>,
}

impl Let {
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

    pub(crate) fn find_free_variables(&self, initialized: bool) -> HashSet<String> {
        let mut all_variables = HashSet::new();
        let mut bound_variables = HashSet::<&str>::new();

        for definition in &self.definitions {
            let mut variables = definition.find_free_variables(initialized);

            for bound_variable in &bound_variables {
                variables.remove(*bound_variable);
            }

            all_variables.extend(variables);

            bound_variables.insert(definition.name());
        }

        let mut variables = self.expression.find_free_variables(initialized);

        for bound_variable in &bound_variables {
            variables.remove(*bound_variable);
        }

        all_variables.extend(variables);

        all_variables
    }

    pub(crate) fn infer_environment(&self, variables: &HashMap<String, Type>) -> Self {
        let mut variables = variables.clone();
        let mut definitions = vec![];

        for definition in &self.definitions {
            definitions.push(definition.infer_environment(&variables));
            variables.insert(definition.name().into(), definition.type_().clone().into());
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
