use crate::types::{self, Algebraic, Type};

#[derive(Clone, Debug, PartialEq)]
pub struct Constructor {
    type_: Algebraic,
    index: u64,
}

impl Constructor {
    pub fn new(type_: Algebraic, index: u64) -> Self {
        Self { type_, index }
    }

    pub fn algebraic_type(&self) -> &Algebraic {
        &self.type_
    }

    pub fn constructor_type(&self) -> &types::Constructor {
        &self.type_.constructors()[&self.index]
    }

    pub fn index(&self) -> u64 {
        self.index
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
