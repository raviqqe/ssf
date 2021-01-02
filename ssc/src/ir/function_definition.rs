use super::argument::Argument;
use super::statement::Statement;
use crate::types::Type;

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionDefinition {
    name: String,
    arguments: Vec<Argument>,
    statements: Vec<Statement>,
    result_type: Type,
}

impl FunctionDefinition {
    pub fn new(
        name: impl Into<String>,
        arguments: Vec<Argument>,
        statements: Vec<Statement>,
        result_type: impl Into<Type> + Clone,
    ) -> Self {
        Self {
            name: name.into(),
            arguments,
            statements,
            result_type: result_type.into(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn arguments(&self) -> &[Argument] {
        &self.arguments
    }

    pub fn statements(&self) -> &[Statement] {
        &self.statements
    }

    pub fn result_type(&self) -> &Type {
        &self.result_type
    }
}
