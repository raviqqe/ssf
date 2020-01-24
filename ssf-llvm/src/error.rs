use std::error::Error;
use std::fmt::Display;

#[derive(Clone, Debug, PartialEq)]
pub enum CompileError {
    Analysis(ssf::AnalysisError),
    Llvm(String),
    VariableNotFound,
}

impl Display for CompileError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(formatter, "{:?}", self)
    }
}

impl Error for CompileError {}

impl From<ssf::AnalysisError> for CompileError {
    fn from(error: ssf::AnalysisError) -> Self {
        Self::Analysis(error)
    }
}

impl From<inkwell::support::LLVMString> for CompileError {
    fn from(string: inkwell::support::LLVMString) -> Self {
        Self::Llvm(string.to_string())
    }
}
