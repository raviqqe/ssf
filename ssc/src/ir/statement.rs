use super::atomic_store::AtomicStore;
use super::if_::If;
use super::return_::Return;
use super::store::Store;
use super::switch::Switch;
use super::variable_definition::VariableDefinition;

#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    AtomicStore(AtomicStore),
    If(If),
    Return(Return),
    Store(Store),
    Switch(Switch),
    Unreachable,
    VariableDefinition(VariableDefinition),
}
