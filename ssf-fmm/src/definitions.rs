use crate::entry_functions;
use crate::expressions;
use crate::types;
use crate::utilities;
use crate::variable_builder::VariableBuilder;
use std::collections::HashMap;

pub fn compile_definition(
    module_builder: &fmm::build::ModuleBuilder,
    definition: &ssf::ir::Definition,
    global_variables: &HashMap<String, VariableBuilder>,
) {
    module_builder.define_variable(
        definition.name(),
        utilities::record(vec![
            entry_functions::compile(module_builder, definition, global_variables),
            expressions::compile_arity(definition.arguments().iter().count()).into(),
            fmm::ir::Undefined::new(types::compile_closure_payload(definition)).into(),
        ]),
        definition.is_thunk(),
        true,
    );
}
