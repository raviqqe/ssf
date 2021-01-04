use super::entry_functions;
use super::expressions;
use super::types;

pub fn compile_definition(
    module: &ssc::ir::Module,
    definition: &ssf::ir::Definition,
) -> ssc::ir::Module {
    let closure_type = types::compile_sized_closure(definition);
    let entry_function_definitions = entry_functions::compile(definition);

    ssc::ir::Module::new(
        module.variable_declarations().to_vec(),
        module.function_declarations().to_vec(),
        module
            .variable_definitions()
            .iter()
            .cloned()
            .chain(vec![ssc::ir::VariableDefinition::new(
                definition.name(),
                ssc::ir::Record::new(
                    closure_type,
                    vec![
                        ssc::ir::Variable::new(entry_function_definitions[0].name()).into(),
                        expressions::compile_arity(definition.arguments().iter().count() as u64)
                            .into(),
                        ssc::ir::Expression::Undefined,
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
