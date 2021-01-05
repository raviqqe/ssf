use super::type_::Type;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Union {
    members: Vec<Type>,
}

impl Union {
    pub const fn new(members: Vec<Type>) -> Self {
        Self { members }
    }

    pub fn members(&self) -> &[Type] {
        &self.members
    }
}
