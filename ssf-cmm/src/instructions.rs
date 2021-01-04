use crate::names;

pub fn bitcast(argument: InstructionResult, type_: cmm::types::Type) -> InstructionResult {
    (cmm::ir::Bitcast::new())
}
