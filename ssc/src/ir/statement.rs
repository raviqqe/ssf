use super::return_::Return;
use super::variable_definition::VariableDefinition;

#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    Return(Return),
    Unreachable,
    VariableDefinition(VariableDefinition),
}
