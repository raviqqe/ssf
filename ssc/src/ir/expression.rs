use super::address_calculation::AddressCalculation;
use super::atomic_load::AtomicLoad;
use super::bitcast::Bitcast;
use super::call::Call;
use super::compare_and_swap::CompareAndSwap;
use super::constructor::Constructor;
use super::load::Load;
use super::primitive::Primitive;
use super::primitive_operation::PrimitiveOperation;
use super::variable::Variable;

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    AddressCalculation(AddressCalculation),
    AtomicLoad(AtomicLoad),
    Bitcast(Bitcast),
    Call(Call),
    CompareAndSwap(CompareAndSwap),
    Constructor(Constructor),
    Load(Load),
    Primitive(Primitive),
    PrimitiveOperation(PrimitiveOperation),
    Undefined,
    Variable(Variable),
}

impl From<AddressCalculation> for Expression {
    fn from(calculation: AddressCalculation) -> Self {
        Self::AddressCalculation(calculation)
    }
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

impl From<CompareAndSwap> for Expression {
    fn from(compare_and_swap: CompareAndSwap) -> Self {
        Self::CompareAndSwap(compare_and_swap)
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

impl From<PrimitiveOperation> for Expression {
    fn from(operation: PrimitiveOperation) -> Self {
        Self::PrimitiveOperation(operation)
    }
}

impl From<Variable> for Expression {
    fn from(variable: Variable) -> Self {
        Self::Variable(variable)
    }
}
