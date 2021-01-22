use super::entry_functions;
use super::expressions;
use super::types;
use super::utilities;
use std::collections::HashMap;

pub fn compile_definition(
    module_builder: &fmm::build::ModuleBuilder,
    definition: &ssf::ir::Definition,
    global_variables: &HashMap<String, fmm::build::TypedExpression>,
) {
    module_builder.define_variable(
        definition.name(),
        utilities::record(vec![
            entry_functions::compile(module_builder, definition, global_variables),
            expressions::compile_arity(definition.arguments().iter().count() as u64).into(),
            fmm::ir::Undefined::new(types::compile_closure_payload(definition)).into(),
        ]),
        definition.is_thunk(),
        true,
    );
}
