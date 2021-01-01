use super::algebraic::Algebraic;
use super::array::*;
use super::function::Function;
use super::primitive::Primitive;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Type {
    Algebraic(Algebraic),
    Array(Array),
    Function(Function),
    Index(usize),
    Primitive(Primitive),
}

impl Type {
    pub fn is_primitive(&self) -> bool {
        matches!(self, Self::Primitive(_))
    }

    pub fn into_algebraic(self) -> Option<Algebraic> {
        match self {
            Self::Algebraic(algebraic) => Some(algebraic),
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

impl From<Algebraic> for Type {
    fn from(algebraic: Algebraic) -> Self {
        Self::Algebraic(algebraic)
    }
}

impl From<Array> for Type {
    fn from(array: Array) -> Self {
        Self::Array(array)
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

#[cfg(test)]
mod tests {
    use super::super::primitive::Primitive;
    use super::*;

    #[test]
    fn to_id() {
        assert_eq!(
            &Type::from(Function::new(Primitive::Float64, Primitive::Float64)).to_id(),
            "(Float64->Float64)"
        );
    }
}
