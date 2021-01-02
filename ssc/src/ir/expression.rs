use super::call::Call;
use super::constructor::Constructor;

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    Call(Call),
    Constructor(Constructor),
    Unreachable,
}
