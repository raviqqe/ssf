use std::error::Error;
use std::fmt::Display;

#[derive(Clone, Debug, PartialEq)]
pub enum CompileError {
    AnalysisError(ssf::AnalysisError),
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

impl From<ssf::TypeCheckError> for CompileError {
    fn from(_: ssf::TypeCheckError) -> Self {
        CompileError::Verification
    }
}

impl From<ssf::AnalysisError> for CompileError {
    fn from(error: ssf::AnalysisError) -> Self {
        Self::AnalysisError(error)
    }
}

impl From<inkwell::support::LLVMString> for CompileError {
    fn from(string: inkwell::support::LLVMString) -> Self {
        Self::LlvmError(string.to_string())
    }
}
