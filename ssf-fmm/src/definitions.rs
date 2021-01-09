use super::entry_functions;
use super::expressions;
use super::types;

pub fn compile_definition(
    module: &fmm::ir::Module,
    definition: &ssf::ir::Definition,
) -> fmm::ir::Module {
    let closure_type = types::compile_sized_closure(definition);
    let entry_function_definitions = entry_functions::compile(definition);

    fmm::ir::Module::new(
        module.variable_declarations().to_vec(),
        module.function_declarations().to_vec(),
        module
            .variable_definitions()
            .iter()
            .cloned()
            .chain(vec![fmm::ir::VariableDefinition::new(
                definition.name(),
                fmm::ir::Record::new(
                    closure_type,
                    vec![
                        fmm::ir::Variable::new(entry_function_definitions[0].name()).into(),
                        expressions::compile_arity(definition.arguments().iter().count() as u64)
                            .into(),
                        fmm::ir::Undefined::new(types::compile_unsized_environment()).into(),
                    ],
                ),
                types::compile_sized_closure(definition),
                !definition.is_thunk(),
            )])
            .collect(),
        module
            .function_definitions()
            .iter()
            .cloned()
            .chain(entry_function_definitions)
            .collect(),
    )
}
