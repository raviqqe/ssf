use super::expression::Expression;
use crate::types::Type;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq)]
pub struct DefaultAlternative {
    variable: String,
    expression: Box<Expression>,
}

impl DefaultAlternative {
    pub fn new(variable: impl Into<String>, expression: impl Into<Expression>) -> Self {
        Self {
            variable: variable.into(),
            expression: Box::new(expression.into()),
        }
    }

    pub fn variable(&self) -> &str {
        &self.variable
    }

    pub fn expression(&self) -> &Expression {
        &self.expression
    }

    pub(crate) fn rename_variables(&self, names: &HashMap<String, String>) -> Self {
        let mut names = names.clone();

        names.remove(&self.variable);

        Self {
            variable: self.variable.clone(),
            expression: self.expression.rename_variables(&names).into(),
        }
    }

    pub(crate) fn find_free_variables(&self, initialized: bool) -> HashSet<String> {
        let mut variables = self.expression.find_free_variables(initialized);

        variables.remove(&self.variable);

        variables
    }

    pub(crate) fn infer_environment(&self, variables: &HashMap<String, Type>) -> Self {
        Self {
            variable: self.variable.clone(),
            expression: self.expression.infer_environment(variables).into(),
        }
    }

    pub(crate) fn convert_types(&self, convert: &impl Fn(&Type) -> Type) -> Self {
        Self {
            variable: self.variable.clone(),
            expression: self.expression.convert_types(convert).into(),
        }
    }
}
