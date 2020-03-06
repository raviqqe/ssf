use super::algebraic::Algebraic;
use super::primitive::Primitive;

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

    pub(crate) fn unfold(&self, algebraic_type: &Algebraic) -> Self {
        match self {
            Self::Algebraic(other) => other.unfold_with(algebraic_type).into(),
            Self::Index(index) => {
                if *index == 0 {
                    algebraic_type.clone().into()
                } else {
                    Self::Index(index - 1)
                }
            }
            Self::Primitive(primitive) => primitive.clone().into(),
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
