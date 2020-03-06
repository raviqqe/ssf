use super::expression::Expression;
use crate::types::{self, Type};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq)]
pub struct Bitcast {
    expression: Box<Expression>,
    type_: types::Value,
}

impl Bitcast {
    pub fn new(expression: impl Into<Expression>, type_: impl Into<types::Value>) -> Self {
        Self {
            expression: Box::new(expression.into()),
            type_: type_.into(),
        }
    }

    pub fn expression(&self) -> &Expression {
        &self.expression
    }

    pub fn type_(&self) -> &types::Value {
        &self.type_
    }

    pub(crate) fn rename_variables(&self, names: &HashMap<String, String>) -> Self {
        Self::new(self.expression.rename_variables(names), self.type_.clone())
    }

    pub(crate) fn find_variables(&self, excluded_variables: &HashSet<String>) -> HashSet<String> {
        self.expression.find_variables(excluded_variables)
    }

    pub(crate) fn infer_environment(
        &self,
        variables: &HashMap<String, Type>,
        global_variables: &HashSet<String>,
    ) -> Self {
        Self::new(
            self.expression
                .infer_environment(variables, global_variables),
            self.type_.clone(),
        )
    }

    pub(crate) fn convert_types(&self, convert: &impl Fn(&Type) -> Type) -> Self {
        Self::new(
            self.expression.convert_types(convert),
            convert(&self.type_.clone().into()).into_value().unwrap(),
        )
    }
}
