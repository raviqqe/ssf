use super::if_::If;
use super::return_::Return;
use super::switch::Switch;
use super::variable_definition::VariableDefinition;

#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    If(If),
    Return(Return),
    Switch(Switch),
    Unreachable,
    VariableDefinition(VariableDefinition),
}
