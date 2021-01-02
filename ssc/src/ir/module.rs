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
        function_declarations: Vec<FunctionDeclaration>,
        variable_declarations: Vec<VariableDeclaration>,
        function_definitions: Vec<FunctionDefinition>,
        variable_definitions: Vec<VariableDefinition>,
    ) -> Self {
        Self {
            function_declarations,
            variable_declarations,
            function_definitions,
            variable_definitions,
        }
    }

    pub fn function_declarations(&self) -> &[FunctionDeclaration] {
        &self.function_declarations
    }

    pub fn function_definitions(&self) -> &[FunctionDefinition] {
        &self.function_definitions
    }
}
