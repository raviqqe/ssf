use super::expression::Expression;
use super::variable::Variable;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq)]
pub struct Application {
    function: Variable,
    arguments: Vec<Expression>,
}

impl Application {
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

    pub(crate) fn find_variables(&self, excluded_variables: &HashSet<String>) -> HashSet<String> {
        self.arguments.iter().fold(
            self.function.find_variables(excluded_variables),
            |mut variables, argument| {
                variables.extend(argument.find_variables(excluded_variables));
                variables
            },
        )
    }
}
