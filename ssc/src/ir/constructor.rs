use super::expression::Expression;
use crate::types;

#[derive(Clone, Debug, PartialEq)]
pub struct Constructor {
    type_: types::Constructor,
    elements: Vec<Expression>,
}

impl Constructor {
    pub fn new(type_: types::Constructor, elements: Vec<Expression>) -> Self {
        Self { type_, elements }
    }

    pub fn type_(&self) -> &types::Constructor {
        &self.type_
    }

    pub fn elements(&self) -> &[Expression] {
        &self.elements
    }
}
