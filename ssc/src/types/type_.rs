use super::constructor::Constructor;
use super::function::Function;
use super::pointer::Pointer;
use super::primitive::Primitive;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Type {
    Constructor(Constructor),
    Function(Function),
    Primitive(Primitive),
    Pointer(Pointer),
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

impl From<Pointer> for Type {
    fn from(pointer: Pointer) -> Self {
        Self::Pointer(pointer)
    }
}
