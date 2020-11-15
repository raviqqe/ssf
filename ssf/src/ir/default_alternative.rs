use super::expression::Expression;
use crate::types::Type;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct DefaultAlternative {
    variable: String,
    expression: Arc<Expression>,
}

impl DefaultAlternative {
    pub fn new(variable: impl Into<String>, expression: impl Into<Expression>) -> Self {
        Self {
            variable: variable.into(),
            expression: Arc::new(expression.into()),
        }
    }

    pub fn variable(&self) -> &str {
        &self.variable
    }

    pub fn expression(&self) -> &Expression {
        &self.expression
    }

    pub(crate) fn find_variables(&self) -> HashSet<String> {
        let mut variables = self.expression.find_variables();

        variables.remove(&self.variable);

        variables
    }

    pub(crate) fn infer_environment(
        &self,
        type_: impl Into<Type>,
        variables: &HashMap<String, Type>,
    ) -> Self {
        let mut variables = variables.clone();

        variables.insert(self.variable.clone(), type_.into());

        Self {
            variable: self.variable.clone(),
            expression: self.expression.infer_environment(&variables).into(),
        }
    }

    pub(crate) fn convert_types(&self, convert: &impl Fn(&Type) -> Type) -> Self {
        Self {
            variable: self.variable.clone(),
            expression: self.expression.convert_types(convert).into(),
        }
    }
}
