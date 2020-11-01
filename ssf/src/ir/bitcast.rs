use super::expression::Expression;
use crate::types::Type;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq)]
pub struct Bitcast {
    expression: Box<Expression>,
    type_: Type,
}

impl Bitcast {
    pub fn new(expression: impl Into<Expression>, type_: impl Into<Type>) -> Self {
        Self {
            expression: Box::new(expression.into()),
            type_: type_.into(),
        }
    }

    pub fn expression(&self) -> &Expression {
        &self.expression
    }

    pub fn type_(&self) -> &Type {
        &self.type_
    }

    pub(crate) fn rename_variables(&self, names: &HashMap<String, String>) -> Self {
        Self::new(self.expression.rename_variables(names), self.type_.clone())
    }

    pub(crate) fn find_free_variables(&self) -> HashSet<String> {
        self.expression.find_free_variables()
    }

    pub(crate) fn infer_environment(&self, variables: &HashMap<String, Type>) -> Self {
        Self::new(
            self.expression.infer_environment(variables),
            self.type_.clone(),
        )
    }

    pub(crate) fn convert_types(&self, convert: &impl Fn(&Type) -> Type) -> Self {
        Self::new(
            self.expression.convert_types(convert),
            convert(&self.type_.clone().into()),
        )
    }
}
