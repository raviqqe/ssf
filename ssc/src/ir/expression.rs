use super::bitcast::Bitcast;
use super::call::Call;
use super::constructor::Constructor;
use super::primitive::Primitive;

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    Bitcast(Bitcast),
    Call(Call),
    Constructor(Constructor),
    Primitive(Primitive),
}
