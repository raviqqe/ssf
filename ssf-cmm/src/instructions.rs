use crate::instruction_result::InstructionResult;
use crate::names;

pub fn bitcast(argument: InstructionResult, type_: cmm::types::Type) -> InstructionResult {
    let name = names::generate_name();

    InstructionResult::new(
        argument
            .statements()
            .iter()
            .cloned()
            .chain(vec![cmm::ir::Bitcast::new(
                argument.variable().clone(),
                type_,
                &name,
            )])
            .collect(),
        name,
    )
}
