use super::expression;
use super::type_;

const ENVIRONMENT_NAME: &str = "_environment";

pub fn compile(definition: &ssf::ir::Definition) -> Vec<ssc::ir::FunctionDefinition> {
    if definition.is_thunk() {
        compile_thunk(definition)
    } else {
        vec![compile_non_thunk(definition)]
    }
}

fn compile_non_thunk(definition: &ssf::ir::Definition) -> ssc::ir::FunctionDefinition {
    ssc::ir::FunctionDefinition::new(
        generate_closure_entry_name(definition.name()),
        compile_arguments(definition),
        {
            let (statements, result) = compile_body(definition);

            statements
                .into_iter()
                .chain(vec![ssc::ir::Return::new(result).into()])
                .collect()
        },
        type_::compile(definition.result_type()),
    )
}

fn compile_thunk(definition: &ssf::ir::Definition) -> Vec<ssc::ir::FunctionDefinition> {
    const ENTRY_POINTER_NAME: &str = "_entry_pointer";

    let entry_function_name = generate_closure_entry_name(definition.name());
    let normal_entry_function_definition = compile_normal_entry(definition);
    let locked_entry_function_definition = compile_locked_entry(definition);
    let arguments = compile_arguments(definition);

    vec![
        ssc::ir::FunctionDefinition::new(
            &entry_function_name,
            arguments.clone(),
            vec![
                ssc::ir::Assignment::new(
                    ENTRY_POINTER_NAME,
                    compile_entry_pointer(&type_::compile_entry_function_from_definition(
                        definition,
                    )),
                )
                .into(),
                ssc::ir::If::new(
                    ssc::ir::CompareAndSwap::new(
                        ssc::ir::Variable::new(ENTRY_POINTER_NAME),
                        ssc::ir::Variable::new(&entry_function_name),
                        ssc::ir::Variable::new(locked_entry_function_definition.name()),
                    ),
                    {
                        let (statements, result) = compile_body(definition);

                        statements
                            .into_iter()
                            .chain(vec![
                                ssc::ir::Store::new(
                                    result.clone(),
                                    ssc::ir::Bitcast::new(
                                        compile_environment_pointer(),
                                        ssc::types::Pointer::new(type_::compile(
                                            definition.result_type(),
                                        )),
                                    ),
                                )
                                .into(),
                                ssc::ir::AtomicStore::new(
                                    ssc::ir::Variable::new(normal_entry_function_definition.name()),
                                    ssc::ir::Variable::new(ENTRY_POINTER_NAME),
                                )
                                .into(),
                                ssc::ir::Return::new(result).into(),
                            ])
                            .collect()
                    },
                    vec![ssc::ir::Return::new(ssc::ir::Call::new(
                        ssc::ir::AtomicLoad::new(ssc::ir::Variable::new(ENTRY_POINTER_NAME)),
                        arguments
                            .iter()
                            .map(|argument| ssc::ir::Variable::new(argument.name()).into())
                            .collect(),
                    ))
                    .into()],
                )
                .into(),
            ],
            type_::compile(definition.result_type()),
        ),
        normal_entry_function_definition,
        locked_entry_function_definition,
    ]
}

fn compile_body(definition: &ssf::ir::Definition) -> (Vec<ssc::ir::Statement>, ssc::ir::Variable) {
    let (statements, variable) = expression::compile(definition.body());

    (
        definition
            .environment()
            .iter()
            .enumerate()
            .map(|(index, free_variable)| {
                ssc::ir::Assignment::new(
                    free_variable.name(),
                    ssc::ir::Load::new(ssc::ir::AddressCalculation::new(
                        ssc::ir::Bitcast::new(
                            compile_environment_pointer(),
                            ssc::types::Pointer::new(type_::compile_environment(definition)),
                        ),
                        vec![
                            ssc::ir::Primitive::PointerInteger(0).into(),
                            ssc::ir::Primitive::PointerInteger(index as u64).into(),
                        ],
                    )),
                )
                .into()
            })
            .chain(statements)
            .collect(),
        variable,
    )
}

fn compile_normal_entry(definition: &ssf::ir::Definition) -> ssc::ir::FunctionDefinition {
    ssc::ir::FunctionDefinition::new(
        generate_normal_entry_name(definition.name()),
        compile_arguments(definition),
        compile_normal_body(definition),
        type_::compile(definition.result_type()),
    )
}

fn compile_locked_entry(definition: &ssf::ir::Definition) -> ssc::ir::FunctionDefinition {
    let entry_function_name = generate_locked_entry_name(definition.name());

    ssc::ir::FunctionDefinition::new(
        &entry_function_name,
        compile_arguments(definition),
        vec![ssc::ir::If::new(
            ssc::ir::PrimitiveOperation::new(
                ssc::ir::PrimitiveOperator::Equal,
                ssc::ir::AtomicLoad::new(compile_entry_pointer(
                    &type_::compile_entry_function_from_definition(definition),
                )),
                ssc::ir::Variable::new(&entry_function_name),
            ),
            vec![ssc::ir::Statement::Unreachable],
            compile_normal_body(definition),
        )
        .into()],
        type_::compile(definition.result_type()),
    )
}

fn compile_normal_body(definition: &ssf::ir::Definition) -> Vec<ssc::ir::Statement> {
    vec![
        ssc::ir::Return::new(ssc::ir::Load::new(ssc::ir::Bitcast::new(
            compile_environment_pointer(),
            ssc::types::Pointer::new(type_::compile(definition.result_type())),
        )))
        .into(),
    ]
}

fn compile_entry_pointer(entry_function_type: &ssc::types::Function) -> ssc::ir::Expression {
    ssc::ir::AddressCalculation::new(
        ssc::ir::Bitcast::new(
            compile_environment_pointer(),
            ssc::types::Pointer::new(ssc::types::Pointer::new(entry_function_type.clone())),
        ),
        vec![ssc::ir::Primitive::PointerInteger(-2i64 as u64).into()],
    )
    .into()
}

fn compile_environment_pointer() -> ssc::ir::Variable {
    ssc::ir::Variable::new(ENVIRONMENT_NAME)
}

fn compile_arguments(definition: &ssf::ir::Definition) -> Vec<ssc::ir::Argument> {
    vec![ssc::ir::Argument::new(
        ENVIRONMENT_NAME,
        ssc::types::Pointer::new(type_::compile_environment(definition)),
    )]
    .into_iter()
    .chain(
        definition.arguments().iter().map(|argument| {
            ssc::ir::Argument::new(argument.name(), type_::compile(argument.type_()))
        }),
    )
    .collect()
}

fn generate_closure_entry_name(name: &str) -> String {
    [name, "_entry"].concat()
}

fn generate_normal_entry_name(name: &str) -> String {
    [name, "_entry_normal"].concat()
}

fn generate_locked_entry_name(name: &str) -> String {
    [name, "_entry_locked"].concat()
}
