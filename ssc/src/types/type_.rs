use super::array::Array;
use super::constructor::Constructor;
use super::function::Function;
use super::pointer::Pointer;
use super::primitive::Primitive;
use super::union::Union;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Type {
    Array(Array),
    Constructor(Constructor),
    Function(Function),
    Primitive(Primitive),
    Pointer(Pointer),
    Union(Union),
}

impl From<Array> for Type {
    fn from(array: Array) -> Self {
        Self::Array(array)
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

impl From<Pointer> for Type {
    fn from(pointer: Pointer) -> Self {
        Self::Pointer(pointer)
    }
}

impl From<Union> for Type {
    fn from(union: Union) -> Self {
        Self::Union(union)
    }
}
