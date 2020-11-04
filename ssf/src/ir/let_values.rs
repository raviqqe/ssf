use super::expression::Expression;
use super::value_definition::ValueDefinition;
use crate::types::Type;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq)]
pub struct LetValues {
    definitions: Vec<ValueDefinition>,
    expression: Box<Expression>,
}

impl LetValues {
    pub fn new(definitions: Vec<ValueDefinition>, expression: impl Into<Expression>) -> Self {
        Self {
            definitions,
            expression: Box::new(expression.into()),
        }
    }

    pub fn definitions(&self) -> &[ValueDefinition] {
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

    pub(crate) fn find_variables(&self) -> HashSet<String> {
        let mut all_variables = HashSet::new();
        let mut bound_variables = HashSet::<&str>::new();

        for definition in &self.definitions {
            let mut variables = definition.find_variables();

            for bound_variable in &bound_variables {
                variables.remove(*bound_variable);
            }

            all_variables.extend(variables);

            bound_variables.insert(definition.name());
        }

        let mut variables = self.expression.find_variables();

        for bound_variable in &bound_variables {
            variables.remove(*bound_variable);
        }

        all_variables
    }

    pub(crate) fn infer_environment(&self, variables: &HashMap<String, Type>) -> Self {
        let mut variables = variables.clone();
        let mut definitions = vec![];

        for value_definition in &self.definitions {
            definitions.push(value_definition.infer_environment(&variables));
            variables.insert(
                value_definition.name().into(),
                value_definition.type_().clone().into(),
            );
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
