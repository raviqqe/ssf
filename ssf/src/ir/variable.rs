use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq)]
pub struct Variable {
    name: String,
}

impl Variable {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn find_variables(&self) -> HashSet<String> {
        vec![self.name.clone()].into_iter().collect()
    }
}
