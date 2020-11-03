use super::definition::Definition;
use super::expression::Expression;
use crate::types::Type;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct Let {
    definition: Arc<Definition>,
    expression: Arc<Expression>,
}

impl Let {
    pub fn new(definition: Definition, expression: impl Into<Expression>) -> Self {
        Self {
            definition: definition.into(),
            expression: expression.into().into(),
        }
    }

    pub fn definition(&self) -> &Definition {
        &self.definition
    }

    pub fn expression(&self) -> &Expression {
        &self.expression
    }

    pub(crate) fn rename_variables(&self, names: &HashMap<String, String>) -> Self {
        Self::new(self.definition.rename_variables(&names), {
            let mut names = names.clone();
            names.remove(self.definition.name());
            self.expression.rename_variables(&names)
        })
    }

    pub(crate) fn find_free_variables(&self, initialized: bool) -> HashSet<String> {
        self.definition
            .find_free_variables(initialized)
            .into_iter()
            .chain({
                let mut variables = self.expression.find_free_variables(initialized);

                variables.remove(self.definition.name());

                variables
            })
            .collect()
    }

    pub(crate) fn infer_environment(&self, variables: &HashMap<String, Type>) -> Self {
        Self::new(self.definition.infer_environment(&variables), {
            let mut variables = variables.clone();

            variables.insert(
                self.definition.name().into(),
                self.definition.type_().clone().into(),
            );

            self.expression.infer_environment(&variables)
        })
    }

    pub(crate) fn convert_types(&self, convert: &impl Fn(&Type) -> Type) -> Self {
        Self::new(
            self.definition.convert_types(convert),
            self.expression.convert_types(convert),
        )
    }
}
