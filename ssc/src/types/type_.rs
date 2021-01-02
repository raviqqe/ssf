use super::constructor::Constructor;
use super::function::Function;
use super::primitive::Primitive;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Type {
    Constructor(Constructor),
    Function(Function),
    Primitive(Primitive),
}

impl Type {
    pub fn is_primitive(&self) -> bool {
        matches!(self, Self::Primitive(_))
    }

    pub fn into_constructor(self) -> Option<Constructor> {
        match self {
            Self::Constructor(constructor) => Some(constructor),
            _ => None,
        }
    }

    pub fn into_function(self) -> Option<Function> {
        match self {
            Self::Function(function) => Some(function),
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

impl From<Constructor> for Type {
    fn from(constructor: Constructor) -> Self {
        Self::Constructor(constructor)
    }
}

impl From<Function> for Type {
    fn from(function: Function) -> Self {
        Self::Function(function)
    }
}

impl From<Primitive> for Type {
    fn from(primitive: Primitive) -> Self {
        Self::Primitive(primitive)
    }
}
