use crate::types::{self, Type};

#[derive(Clone, Debug, PartialEq)]
pub struct Declaration {
    name: String,
    type_: types::Function,
}

impl Declaration {
    pub fn new(name: impl Into<String>, type_: types::Function) -> Self {
        Self {
            name: name.into(),
            type_,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn type_(&self) -> &types::Function {
        &self.type_
    }

    pub(crate) fn convert_types(&self, convert: &impl Fn(&Type) -> Type) -> Self {
        Self {
            name: self.name.clone(),
            type_: convert(&self.type_.clone().into()).into_function().unwrap(),
        }
    }
}
