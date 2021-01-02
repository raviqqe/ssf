use super::atomic_load::AtomicLoad;
use super::bitcast::Bitcast;
use super::call::Call;
use super::constructor::Constructor;
use super::load::Load;
use super::primitive::Primitive;

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    AtomicLoad(AtomicLoad),
    Bitcast(Bitcast),
    Call(Call),
    Constructor(Constructor),
    Load(Load),
    Primitive(Primitive),
    Undefined,
}
