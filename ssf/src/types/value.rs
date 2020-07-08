use super::algebraic::Algebraic;
use super::primitive::Primitive;

/// Value types are ones which are not functions.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Value {
    Algebraic(Algebraic),
    Index(usize),
    Primitive(Primitive),
}

impl Value {
    pub fn to_id(&self) -> String {
        match self {
            Self::Algebraic(algebraic) => algebraic.to_id(),
            Self::Index(index) => format!("{}", index),
            Self::Primitive(primitive) => primitive.to_id(),
        }
    }

    pub fn into_algebraic(self) -> Option<Algebraic> {
        match self {
            Self::Algebraic(algebraic) => Some(algebraic),
            _ => None,
        }
    }

    pub fn into_primitive(self) -> Option<Primitive> {
        match self {
            Self::Primitive(primitive) => Some(primitive),
            _ => None,
        }
    }
}

impl From<Algebraic> for Value {
    fn from(algebraic: Algebraic) -> Self {
        Self::Algebraic(algebraic)
    }
}

impl From<Primitive> for Value {
    fn from(primitive: Primitive) -> Self {
        Self::Primitive(primitive)
    }
}
