use super::expression::Expression;
use super::variable::Variable;
use crate::types::Type;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionApplication {
    function: Variable,
    arguments: Vec<Expression>,
}

impl FunctionApplication {
    pub fn new(function: Variable, arguments: Vec<Expression>) -> Self {
        Self {
            function,
            arguments,
        }
    }

    pub fn function(&self) -> &Variable {
        &self.function
    }

    pub fn arguments(&self) -> &[Expression] {
        &self.arguments
    }

    pub(crate) fn rename_variables(&self, names: &HashMap<String, String>) -> Self {
        Self::new(
            self.function.rename_variables(names),
            self.arguments
                .iter()
                .map(|argument| argument.rename_variables(names))
                .collect(),
        )
    }

    pub(crate) fn find_variables(&self) -> HashSet<String> {
        let mut variables = self.function.find_variables();

        for argument in &self.arguments {
            variables.extend(argument.find_variables());
        }

        variables
    }

    pub(crate) fn infer_environment(&self, variables: &HashMap<String, Type>) -> Self {
        Self::new(
            // TODO Infer environment for function expressions.
            self.function.clone(),
            self.arguments
                .iter()
                .map(|argument| argument.infer_environment(variables))
                .collect(),
        )
    }

    pub(crate) fn convert_types(&self, convert: &impl Fn(&Type) -> Type) -> Self {
        Self {
            function: self.function.clone(),
            arguments: self
                .arguments
                .iter()
                .map(|argument| argument.convert_types(convert))
                .collect(),
        }
    }
}
