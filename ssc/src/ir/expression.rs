use super::primitive::Primitive;
use super::record::Record;
use super::union::Union;
use super::variable::Variable;

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    Primitive(Primitive),
    Record(Record),
    Undefined,
    Union(Union),
    Variable(Variable),
}

impl From<Record> for Expression {
    fn from(record: Record) -> Self {
        Self::Record(record)
    }
}

impl From<Primitive> for Expression {
    fn from(primitive: Primitive) -> Self {
        Self::Primitive(primitive)
    }
}

impl From<Union> for Expression {
    fn from(union: Union) -> Self {
        Self::Union(union)
    }
}

impl From<Variable> for Expression {
    fn from(variable: Variable) -> Self {
        Self::Variable(variable.into())
    }
}
