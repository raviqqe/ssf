use super::constructor::Constructor;
use super::expression::Expression;
use crate::types::Type;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq)]
pub struct ConstructorApplication {
    constructor: Constructor,
    arguments: Vec<Expression>,
}

impl ConstructorApplication {
    pub fn new(constructor: Constructor, arguments: Vec<Expression>) -> Self {
        Self {
            constructor,
            arguments,
        }
    }

    pub fn constructor(&self) -> &Constructor {
        &self.constructor
    }

    pub fn arguments(&self) -> &[Expression] {
        &self.arguments
    }

    pub(crate) fn rename_variables(&self, names: &HashMap<String, String>) -> Self {
        Self::new(
            self.constructor.clone(),
            self.arguments
                .iter()
                .map(|argument| argument.rename_variables(names))
                .collect(),
        )
    }

    pub(crate) fn find_free_variables(&self) -> HashSet<String> {
        let mut variables = HashSet::new();

        for argument in &self.arguments {
            variables.extend(argument.find_free_variables());
        }

        variables
    }

    pub(crate) fn infer_environment(&self, variables: &HashMap<String, Type>) -> Self {
        Self::new(
            self.constructor.clone(),
            self.arguments
                .iter()
                .map(|argument| argument.infer_environment(variables))
                .collect(),
        )
    }

    pub(crate) fn convert_types(&self, convert: &impl Fn(&Type) -> Type) -> Self {
        Self::new(
            self.constructor.convert_types(convert),
            self.arguments
                .iter()
                .map(|argument| argument.convert_types(convert))
                .collect(),
        )
    }
}
