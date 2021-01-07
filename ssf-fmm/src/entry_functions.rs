use super::expressions;
use super::types;

const ENVIRONMENT_NAME: &str = "_environment";

pub fn compile(definition: &ssf::ir::Definition) -> Vec<fmm::ir::FunctionDefinition> {
    if definition.is_thunk() {
        compile_thunk(definition)
    } else {
        vec![compile_non_thunk(definition)]
    }
}

fn compile_non_thunk(definition: &ssf::ir::Definition) -> fmm::ir::FunctionDefinition {
    fmm::ir::FunctionDefinition::new(
        generate_closure_entry_name(definition.name()),
        compile_arguments(definition),
        {
            let (instructions, result) = compile_body(definition);

            instructions
                .into_iter()
                .chain(vec![fmm::ir::Return::new(result).into()])
                .collect()
        },
        types::compile(definition.result_type()),
    )
}

fn compile_thunk(definition: &ssf::ir::Definition) -> Vec<fmm::ir::FunctionDefinition> {
    const ENTRY_POINTER_NAME: &str = "_entry_pointer";

    let entry_function_name = generate_closure_entry_name(definition.name());
    let normal_entry_function_definition = compile_normal_entry(definition);
    let locked_entry_function_definition = compile_locked_entry(definition);
    let arguments = compile_arguments(definition);

    vec![
        fmm::ir::FunctionDefinition::new(
            &entry_function_name,
            arguments.clone(),
            vec![
                fmm::ir::Assignment::new(
                    ENTRY_POINTER_NAME,
                    compile_entry_pointer(&types::compile_entry_function_from_definition(
                        definition,
                    )),
                )
                .into(),
                fmm::ir::If::new(
                    fmm::ir::CompareAndSwap::new(
                        fmm::ir::Variable::new(ENTRY_POINTER_NAME),
                        fmm::ir::Variable::new(&entry_function_name),
                        fmm::ir::Variable::new(locked_entry_function_definition.name()),
                    ),
                    {
                        let (instructions, result) = compile_body(definition);

                        instructions
                            .into_iter()
                            .chain(vec![
                                fmm::ir::Store::new(
                                    result.clone(),
                                    fmm::ir::Bitcast::new(
                                        compile_environment_pointer(),
                                        fmm::types::Pointer::new(types::compile(
                                            definition.result_type(),
                                        )),
                                    ),
                                )
                                .into(),
                                fmm::ir::AtomicStore::new(
                                    fmm::ir::Variable::new(normal_entry_function_definition.name()),
                                    fmm::ir::Variable::new(ENTRY_POINTER_NAME),
                                )
                                .into(),
                                fmm::ir::Return::new(result).into(),
                            ])
                            .collect()
                    },
                    vec![fmm::ir::Return::new(fmm::ir::Call::new(
                        fmm::ir::AtomicLoad::new(fmm::ir::Variable::new(ENTRY_POINTER_NAME)),
                        arguments
                            .iter()
                            .map(|argument| fmm::ir::Variable::new(argument.name()).into())
                            .collect(),
                    ))
                    .into()],
                )
                .into(),
            ],
            types::compile(definition.result_type()),
        ),
        normal_entry_function_definition,
        locked_entry_function_definition,
    ]
}

fn compile_body(
    definition: &ssf::ir::Definition,
) -> (Vec<fmm::ir::Instruction>, fmm::ir::Variable) {
    let (instructions, variable) = expressions::compile(definition.body());

    (
        definition
            .environment()
            .iter()
            .enumerate()
            .map(|(index, free_variable)| {
                fmm::ir::Assignment::new(
                    free_variable.name(),
                    fmm::ir::Load::new(fmm::ir::AddressCalculation::new(
                        fmm::ir::Bitcast::new(
                            compile_environment_pointer(),
                            fmm::types::Pointer::new(types::compile_environment(definition)),
                        ),
                        vec![
                            fmm::ir::Primitive::PointerInteger(0).into(),
                            fmm::ir::Primitive::PointerInteger(index as u64).into(),
                        ],
                    )),
                )
                .into()
            })
            .chain(instructions)
            .collect(),
        variable,
    )
}

fn compile_normal_entry(definition: &ssf::ir::Definition) -> fmm::ir::FunctionDefinition {
    fmm::ir::FunctionDefinition::new(
        generate_normal_entry_name(definition.name()),
        compile_arguments(definition),
        compile_normal_body(definition),
        types::compile(definition.result_type()),
    )
}

fn compile_locked_entry(definition: &ssf::ir::Definition) -> fmm::ir::FunctionDefinition {
    let entry_function_name = generate_locked_entry_name(definition.name());

    fmm::ir::FunctionDefinition::new(
        &entry_function_name,
        compile_arguments(definition),
        vec![fmm::ir::If::new(
            fmm::ir::PrimitiveOperation::new(
                fmm::ir::PrimitiveOperator::Equal,
                fmm::ir::AtomicLoad::new(compile_entry_pointer(
                    &types::compile_entry_function_from_definition(definition),
                )),
                fmm::ir::Variable::new(&entry_function_name),
            ),
            vec![fmm::ir::Instruction::Unreachable],
            compile_normal_body(definition),
        )
        .into()],
        types::compile(definition.result_type()),
    )
}

fn compile_normal_body(definition: &ssf::ir::Definition) -> Vec<fmm::ir::Instruction> {
    vec![
        fmm::ir::Return::new(fmm::ir::Load::new(fmm::ir::Bitcast::new(
            compile_environment_pointer(),
            fmm::types::Pointer::new(types::compile(definition.result_type())),
        )))
        .into(),
    ]
}

fn compile_entry_pointer(entry_function_type: &fmm::types::Function) -> fmm::ir::Expression {
    fmm::ir::AddressCalculation::new(
        fmm::ir::Bitcast::new(
            compile_environment_pointer(),
            fmm::types::Pointer::new(fmm::types::Pointer::new(entry_function_type.clone())),
        ),
        vec![fmm::ir::Primitive::PointerInteger(-2i64 as u64).into()],
    )
    .into()
}

fn compile_environment_pointer() -> fmm::ir::Variable {
    fmm::ir::Variable::new(ENVIRONMENT_NAME)
}

fn compile_arguments(definition: &ssf::ir::Definition) -> Vec<fmm::ir::Argument> {
    vec![fmm::ir::Argument::new(
        ENVIRONMENT_NAME,
        fmm::types::Pointer::new(types::compile_environment(definition)),
    )]
    .into_iter()
    .chain(
        definition.arguments().iter().map(|argument| {
            fmm::ir::Argument::new(argument.name(), types::compile(argument.type_()))
        }),
    )
    .collect()
}

pub fn generate_closure_entry_name(name: &str) -> String {
    [name, "_entry"].concat()
}

fn generate_normal_entry_name(name: &str) -> String {
    [name, "_entry_normal"].concat()
}

fn generate_locked_entry_name(name: &str) -> String {
    [name, "_entry_locked"].concat()
}
