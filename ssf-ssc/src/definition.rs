use super::expression;
use super::type_;

pub fn compile_definition(
    module: &ssc::ir::Module,
    definition: &ssf::ir::Definition,
) -> ssc::ir::Module {
    let closure_type = type_::compile_sized_closure(definition);
    let entry_function_definition = compile_entry_function(definition);

    ssc::ir::Module::new(
        module.variable_declarations().to_vec(),
        module.function_declarations().to_vec(),
        module
            .variable_definitions()
            .iter()
            .cloned()
            .chain(vec![ssc::ir::VariableDefinition::new(
                definition.name(),
                ssc::ir::Constructor::new(
                    closure_type.clone(),
                    vec![
                        ssc::ir::Variable::new(entry_function_definition.name()).into(),
                        expression::compile_arity(
                            definition.arguments().into_iter().count() as u64,
                        )
                        .into(),
                        ssc::ir::Expression::Undefined,
                    ],
                ),
                type_::compile_sized_closure(definition),
                !definition.is_thunk(),
            )])
            .collect(),
        module
            .function_definitions()
            .iter()
            .cloned()
            .chain(vec![entry_function_definition])
            .collect(),
    )
}

fn compile_entry_function(definition: &ssf::ir::Definition) -> ssc::ir::FunctionDefinition {
    ssc::ir::FunctionDefinition::new(
        format!("{}_entry", definition.name()),
        definition
            .arguments()
            .iter()
            .map(|argument| {
                ssc::ir::Argument::new(argument.name(), type_::compile(argument.type_()))
            })
            .collect(),
        todo!(),
        type_::compile(definition.result_type()),
    )
}
