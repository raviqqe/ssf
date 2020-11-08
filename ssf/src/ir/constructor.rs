use crate::types::{self, Algebraic, Type};

#[derive(Clone, Debug, PartialEq)]
pub struct Constructor {
    type_: Algebraic,
    unfolded_type: Algebraic,
    tag: u64,
}

impl Constructor {
    pub fn new(type_: Algebraic, tag: u64) -> Self {
        Self {
            unfolded_type: type_.unfold(),
            type_,
            tag,
        }
    }

    pub fn algebraic_type(&self) -> &Algebraic {
        &self.type_
    }

    pub fn constructor_type(&self) -> &types::Constructor {
        &self.unfolded_type.constructors()[&self.tag]
    }

    pub fn tag(&self) -> u64 {
        self.tag
    }

    pub(crate) fn convert_types(&self, convert: &impl Fn(&Type) -> Type) -> Self {
        Self::new(
            convert(&self.type_.clone().into())
                .into_algebraic()
                .unwrap(),
            self.tag,
        )
    }
}
