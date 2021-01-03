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

impl From<AtomicStore> for Statement {
    fn from(store: AtomicStore) -> Self {
        Self::AtomicStore(store)
    }
}

impl From<If> for Statement {
    fn from(if_: If) -> Self {
        Self::If(if_)
    }
}

impl From<Return> for Statement {
    fn from(return_: Return) -> Self {
        Self::Return(return_)
    }
}

impl From<Store> for Statement {
    fn from(store: Store) -> Self {
        Self::Store(store)
    }
}

impl From<Switch> for Statement {
    fn from(switch: Switch) -> Self {
        Self::Switch(switch)
    }
}

impl From<VariableDefinition> for Statement {
    fn from(definition: VariableDefinition) -> Self {
        Self::VariableDefinition(definition)
    }
}
