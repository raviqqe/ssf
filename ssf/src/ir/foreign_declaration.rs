use crate::types::{self, Type};

#[derive(Clone, Debug, PartialEq)]
pub struct ForeignDeclaration {
    name: String,
    foreign_name: String,
    type_: types::Function,
}

impl ForeignDeclaration {
    pub fn new(
        name: impl Into<String>,
        foreign_name: impl Into<String>,
        type_: types::Function,
    ) -> Self {
        Self {
            name: name.into(),
            foreign_name: foreign_name.into(),
            type_,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn foreign_name(&self) -> &str {
        &self.foreign_name
    }

    pub fn type_(&self) -> &types::Function {
        &self.type_
    }

    pub(crate) fn convert_types(&self, convert: &impl Fn(&Type) -> Type) -> Self {
        Self {
            name: self.name.clone(),
            foreign_name: self.foreign_name.clone(),
            type_: convert(&self.type_.clone().into()).into_function().unwrap(),
        }
    }
}
