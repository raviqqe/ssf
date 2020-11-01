use super::expression::Expression;
use crate::types::Type;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionApplication {
    function: Arc<Expression>,
    argument: Arc<Expression>,
}

impl FunctionApplication {
    pub fn new(function: impl Into<Expression>, argument: impl Into<Expression>) -> Self {
        Self {
            function: function.into().into(),
            argument: argument.into().into(),
        }
    }

    pub fn function(&self) -> &Expression {
        &self.function
    }

    pub fn argument(&self) -> &Expression {
        &self.argument
    }

    pub(crate) fn rename_variables(&self, names: &HashMap<String, String>) -> Self {
        Self::new(
            self.function.rename_variables(names),
            self.argument.rename_variables(names),
        )
    }

    pub(crate) fn find_variables(&self) -> HashSet<String> {
        let mut variables = self.function.find_variables();

        variables.extend(self.argument.find_variables());

        variables
    }

    pub(crate) fn infer_environment(&self, variables: &HashMap<String, Type>) -> Self {
        Self::new(
            self.function.infer_environment(variables),
            self.argument.infer_environment(variables),
        )
    }

    pub(crate) fn convert_types(&self, convert: &impl Fn(&Type) -> Type) -> Self {
        Self::new(
            self.function.convert_types(convert),
            self.argument.convert_types(convert),
        )
    }
}
