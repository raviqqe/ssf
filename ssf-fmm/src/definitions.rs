use super::entry_functions;
use super::expressions;
use super::types;
use super::utilities;

pub fn compile_definition(
    module_builder: &fmm::build::ModuleBuilder,
    definition: &ssf::ir::Definition,
) {
    module_builder.define_variable(
        definition.name(),
        utilities::record(vec![
            entry_functions::compile(module_builder, definition),
            expressions::compile_arity(definition.arguments().iter().count() as u64).into(),
            fmm::ir::Undefined::new(types::compile_unsized_environment()).into(),
        ]),
        !definition.is_thunk(),
        true,
    );
}
