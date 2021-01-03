use super::atomic_load::AtomicLoad;
use super::bitcast::Bitcast;
use super::call::Call;
use super::constructor::Constructor;
use super::load::Load;
use super::primitive::Primitive;
use super::variable::Variable;

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    AtomicLoad(AtomicLoad),
    Bitcast(Bitcast),
    Call(Call),
    Constructor(Constructor),
    Load(Load),
    Primitive(Primitive),
    Undefined,
    Variable(Variable),
}

impl From<AtomicLoad> for Expression {
    fn from(load: AtomicLoad) -> Self {
        Self::AtomicLoad(load)
    }
}

impl From<Bitcast> for Expression {
    fn from(bitcast: Bitcast) -> Self {
        Self::Bitcast(bitcast)
    }
}

impl From<Call> for Expression {
    fn from(call: Call) -> Self {
        Self::Call(call)
    }
}

impl From<Constructor> for Expression {
    fn from(constructor: Constructor) -> Self {
        Self::Constructor(constructor)
    }
}

impl From<Load> for Expression {
    fn from(load: Load) -> Self {
        Self::Load(load)
    }
}

impl From<Primitive> for Expression {
    fn from(primitive: Primitive) -> Self {
        Self::Primitive(primitive)
    }
}

impl From<Variable> for Expression {
    fn from(variable: Variable) -> Self {
        Self::Variable(variable)
    }
}
