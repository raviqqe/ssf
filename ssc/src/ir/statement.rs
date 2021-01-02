use super::if_::If;
use super::return_::Return;
use super::variable_definition::VariableDefinition;

#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    If(If),
    Return(Return),
    Unreachable,
    VariableDefinition(VariableDefinition),
}
