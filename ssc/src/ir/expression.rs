use super::constructor::Constructor;

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    Constructor(Constructor),
    Unreachable,
}
