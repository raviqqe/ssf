use super::type_::Type;
use std::collections::BTreeSet;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Union {
    members: BTreeSet<Type>,
}

impl Union {
    pub const fn new(members: BTreeSet<Type>) -> Self {
        Self { members }
    }

    pub fn members(&self) -> &BTreeSet<Type> {
        &self.members
    }
}
