use super::type_check::TypeCheckError;
use petgraph::algo::Cycle;
use std::error::Error;
use std::fmt::Display;

#[derive(Clone, Debug, PartialEq)]
pub enum AnalysisError {
    CircularInitialization,
    TypeCheck(TypeCheckError),
}

impl Display for AnalysisError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(formatter, "{:#?}", self)
    }
}

impl Error for AnalysisError {}

impl<N> From<Cycle<N>> for AnalysisError {
    fn from(_: Cycle<N>) -> Self {
        Self::CircularInitialization
    }
}

impl From<TypeCheckError> for AnalysisError {
    fn from(error: TypeCheckError) -> Self {
        Self::TypeCheck(error)
    }
}
