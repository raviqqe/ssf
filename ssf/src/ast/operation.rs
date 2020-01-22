use super::expression::Expression;
use super::operator::Operator;
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

    pub(crate) fn find_variables(&self, excluded_variables: &HashSet<String>) -> HashSet<String> {
        let mut variables = self.lhs.find_variables(excluded_variables);
        variables.extend(self.rhs.find_variables(excluded_variables));
        variables
    }
}
