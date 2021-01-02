use super::variable_definition::VariableDefinition;

#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    Unreachable,
    VariableDefinition(VariableDefinition),
}
