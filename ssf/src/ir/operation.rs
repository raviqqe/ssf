use super::expression::Expression;
use super::operator::Operator;
use crate::types::Type;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq)]
pub struct Operation {
    operator: Operator,
    lhs: Box<Expression>,
    rhs: Box<Expression>,
}

impl Operation {
    pub fn new(operator: Operator, lhs: impl Into<Expression>, rhs: impl Into<Expression>) -> Self {
        Self {
            operator,
            lhs: Box::new(lhs.into()),
            rhs: Box::new(rhs.into()),
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

    pub(crate) fn find_free_variables(&self) -> HashSet<String> {
        let mut variables = self.lhs.find_free_variables();

        variables.extend(self.rhs.find_free_variables());

        variables
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
