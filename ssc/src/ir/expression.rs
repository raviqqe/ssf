use super::call::Call;
use super::constructor::Constructor;
use super::primitive::Primitive;

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    Call(Call),
    Constructor(Constructor),
    Primitive(Primitive),
}
