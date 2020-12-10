use std::error::Error;
use std::fmt::Display;

#[derive(Clone, Debug, PartialEq)]
pub enum CompileError {
    InvalidBitcast(String, String),
    Llvm(String),
    VariableNotFound(String),
}

impl Display for CompileError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(formatter, "{:?}", self)
    }
}

impl Error for CompileError {}

impl From<&str> for CompileError {
    fn from(string: &str) -> Self {
        Self::Llvm(string.to_string())
    }
}

impl From<inkwell::support::LLVMString> for CompileError {
    fn from(string: inkwell::support::LLVMString) -> Self {
        Self::Llvm(string.to_string())
    }
}
