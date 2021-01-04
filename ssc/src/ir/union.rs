use super::expression::Expression;
use crate::types;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct Union {
    type_: types::Union,
    index: usize,
    member: Arc<Expression>,
}

impl Union {
    pub fn new(type_: types::Union, index: usize, member: impl Into<Expression>) -> Self {
        Self {
            type_,
            index,
            member: member.into().into(),
        }
    }

    pub fn type_(&self) -> &types::Union {
        &self.type_
    }

    pub fn usize(&self) -> &types::Union {
        &self.type_
    }

    pub fn member(&self) -> &Expression {
        &self.member
    }
}
