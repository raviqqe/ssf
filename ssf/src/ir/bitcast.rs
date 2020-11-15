use super::expression::Expression;
use crate::types::Type;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct Bitcast {
    expression: Arc<Expression>,
    type_: Type,
}

impl Bitcast {
    pub fn new(expression: impl Into<Expression>, type_: impl Into<Type>) -> Self {
        Self {
            expression: Arc::new(expression.into()),
            type_: type_.into(),
        }
    }

    pub fn expression(&self) -> &Expression {
        &self.expression
    }

    pub fn type_(&self) -> &Type {
        &self.type_
    }

    pub(crate) fn find_variables(&self) -> HashSet<String> {
        self.expression.find_variables()
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
            convert(&self.type_.clone()),
        )
    }
}
