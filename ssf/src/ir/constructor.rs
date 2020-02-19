use crate::types::{self, Algebraic, Type};

#[derive(Clone, Debug, PartialEq)]
pub struct Constructor {
    type_: Algebraic,
    index: usize,
}

impl Constructor {
    pub fn new(type_: Algebraic, index: usize) -> Self {
        Self { type_, index }
    }

    pub fn type_(&self) -> &Algebraic {
        &self.type_
    }

    pub fn constructor_type(&self) -> &types::Constructor {
        &self.type_.constructors()[self.index]
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn element_types(&self) -> &[Type] {
        self.type_.constructors()[self.index].elements()
    }

    pub(crate) fn convert_types(&self, convert: &impl Fn(&Type) -> Type) -> Self {
        Self {
            type_: convert(&self.type_.clone().into())
                .into_value()
                .and_then(|value_type| value_type.into_algebraic())
                .unwrap(),
            index: self.index,
        }
    }
}
