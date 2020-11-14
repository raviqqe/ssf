use super::expression::Expression;
use super::operator::Operator;
use crate::types::Type;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct Operation {
    operator: Operator,
    lhs: Arc<Expression>,
    rhs: Arc<Expression>,
}

impl Operation {
    pub fn new(operator: Operator, lhs: impl Into<Expression>, rhs: impl Into<Expression>) -> Self {
        Self {
            operator,
            lhs: Arc::new(lhs.into()),
            rhs: Arc::new(rhs.into()),
        }
    }

    pub fn operator(&self) -> &Operator {
        &self.operator
    }

    pub fn lhs(&self) -> &Expression {
        &self.lhs
    }

    pub fn rhs(&self) -> &Expression {
        &self.rhs
    }

    pub(crate) fn rename_variables(&self, names: &HashMap<String, String>) -> Self {
        Self::new(
            self.operator,
            self.lhs.rename_variables(names),
            self.rhs.rename_variables(names),
        )
    }

    pub(crate) fn find_variables(&self) -> HashSet<String> {
        self.lhs
            .find_variables()
            .into_iter()
            .chain(self.rhs.find_variables())
            .collect()
    }

    pub(crate) fn infer_environment(&self, variables: &HashMap<String, Type>) -> Self {
        Self::new(
            self.operator,
            self.lhs.infer_environment(variables),
            self.rhs.infer_environment(variables),
        )
    }

    pub(crate) fn convert_types(&self, convert: &impl Fn(&Type) -> Type) -> Self {
        Self::new(
            self.operator,
            self.lhs.convert_types(convert),
            self.rhs.convert_types(convert),
        )
    }
}
