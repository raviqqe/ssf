use crate::{ir::*, types::Type};
use std::{error::Error, fmt::Display};

#[derive(Clone, Debug, PartialEq)]
pub enum TypeCheckError {
    ForeignDefinitionNotFound(ForeignDefinition),
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
