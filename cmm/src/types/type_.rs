use super::function::Function;
use super::pointer::Pointer;
use super::primitive::Primitive;
use super::record::Record;
use super::union::Union;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Type {
    Record(Record),
    Function(Function),
    Primitive(Primitive),
    Pointer(Pointer),
    Union(Union),
}

impl From<Record> for Type {
    fn from(record: Record) -> Self {
        Self::Record(record)
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
