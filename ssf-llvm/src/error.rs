use petgraph::algo::Cycle;
use std::error::Error;
use std::fmt::Display;

#[derive(Clone, Debug, PartialEq)]
pub enum CompileError {
    CircularInitialization,
    InvalidTypeIndex,
    LlvmError(String),
    VariableNotFound,
    Verification,
}

impl Display for CompileError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(formatter, "{:?}", self)
    }
}

impl Error for CompileError {}

impl From<ssf::VerificationError> for CompileError {
    fn from(_: ssf::VerificationError) -> Self {
        CompileError::Verification
    }
}

impl<N> From<Cycle<N>> for CompileError {
    fn from(_: Cycle<N>) -> Self {
        Self::CircularInitialization
    }
}

impl From<inkwell::support::LLVMString> for CompileError {
    fn from(string: inkwell::support::LLVMString) -> Self {
        Self::LlvmError(string.to_string())
    }
}
