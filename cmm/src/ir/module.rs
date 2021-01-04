use super::function_declaration::FunctionDeclaration;
use super::function_definition::FunctionDefinition;
use super::variable_declaration::VariableDeclaration;
use super::variable_definition::VariableDefinition;

#[derive(Clone, Debug, PartialEq)]
pub struct Module {
    variable_declarations: Vec<VariableDeclaration>,
    function_declarations: Vec<FunctionDeclaration>,
    variable_definitions: Vec<VariableDefinition>,
    function_definitions: Vec<FunctionDefinition>,
}

impl Module {
    pub fn new(
        variable_declarations: Vec<VariableDeclaration>,
        function_declarations: Vec<FunctionDeclaration>,
        variable_definitions: Vec<VariableDefinition>,
        function_definitions: Vec<FunctionDefinition>,
    ) -> Self {
        Self {
            variable_declarations,
            function_declarations,
            variable_definitions,
            function_definitions,
        }
    }

    pub fn variable_declarations(&self) -> &[VariableDeclaration] {
        &self.variable_declarations
    }

    pub fn function_declarations(&self) -> &[FunctionDeclaration] {
        &self.function_declarations
    }

    pub fn variable_definitions(&self) -> &[VariableDefinition] {
        &self.variable_definitions
    }

    pub fn function_definitions(&self) -> &[FunctionDefinition] {
        &self.function_definitions
    }
}
