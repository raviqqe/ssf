use std::collections::{HashMap, HashSet};

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

    pub(crate) fn rename_variables(&self, names: &HashMap<String, String>) -> Self {
        match names.get(self.name.as_str()) {
            Some(name) => Self::new(name),
            None => self.clone(),
        }
    }

    pub(crate) fn find_free_variables(&self, _: bool) -> HashSet<String> {
        vec![self.name.clone()].into_iter().collect()
    }
}
