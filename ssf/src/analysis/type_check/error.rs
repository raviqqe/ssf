use crate::ir::*;
use crate::types::Type;
use std::error::Error;
use std::fmt::Display;

#[derive(Clone, Debug, PartialEq)]
pub enum TypeCheckError {
    FunctionExpected(Expression),
    NoAlternativeFound(Case),
    TypesNotMatched(Type, Type),
    VariableNotFound(Variable),
    WrongArgumentsLength(Expression),
}

impl Display for TypeCheckError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(formatter, "{:#?}", self)
    }
}

impl Error for TypeCheckError {}
