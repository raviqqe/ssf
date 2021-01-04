use super::expressions;
use super::types;

const ENVIRONMENT_NAME: &str = "_environment";

pub fn compile(definition: &ssf::ir::Definition) -> Vec<cmm::ir::FunctionDefinition> {
    if definition.is_thunk() {
        compile_thunk(definition)
    } else {
        vec![compile_non_thunk(definition)]
    }
}

fn compile_non_thunk(definition: &ssf::ir::Definition) -> cmm::ir::FunctionDefinition {
    cmm::ir::FunctionDefinition::new(
        generate_closure_entry_name(definition.name()),
        compile_arguments(definition),
        {
            let (instructions, result) = compile_body(definition);

            instructions
                .into_iter()
                .chain(vec![cmm::ir::Return::new(result).into()])
                .collect()
        },
        types::compile(definition.result_type()),
    )
}

fn compile_thunk(definition: &ssf::ir::Definition) -> Vec<cmm::ir::FunctionDefinition> {
    const ENTRY_POINTER_NAME: &str = "_entry_pointer";

    let entry_function_name = generate_closure_entry_name(definition.name());
    let normal_entry_function_definition = compile_normal_entry(definition);
    let locked_entry_function_definition = compile_locked_entry(definition);
    let arguments = compile_arguments(definition);

    vec![
        cmm::ir::FunctionDefinition::new(
            &entry_function_name,
            arguments.clone(),
            vec![
                cmm::ir::Assignment::new(
                    ENTRY_POINTER_NAME,
                    compile_entry_pointer(&types::compile_entry_function_from_definition(
                        definition,
                    )),
                )
                .into(),
                cmm::ir::If::new(
                    cmm::ir::CompareAndSwap::new(
                        cmm::ir::Variable::new(ENTRY_POINTER_NAME),
                        cmm::ir::Variable::new(&entry_function_name),
                        cmm::ir::Variable::new(locked_entry_function_definition.name()),
                    ),
                    {
                        let (instructions, result) = compile_body(definition);

                        instructions
                            .into_iter()
                            .chain(vec![
                                cmm::ir::Store::new(
                                    result.clone(),
                                    cmm::ir::Bitcast::new(
                                        compile_environment_pointer(),
                                        cmm::types::Pointer::new(types::compile(
                                            definition.result_type(),
                                        )),
                                    ),
                                )
                                .into(),
                                cmm::ir::AtomicStore::new(
                                    cmm::ir::Variable::new(normal_entry_function_definition.name()),
                                    cmm::ir::Variable::new(ENTRY_POINTER_NAME),
                                )
                                .into(),
                                cmm::ir::Return::new(result).into(),
                            ])
                            .collect()
                    },
                    vec![cmm::ir::Return::new(cmm::ir::Call::new(
                        cmm::ir::AtomicLoad::new(cmm::ir::Variable::new(ENTRY_POINTER_NAME)),
                        arguments
                            .iter()
                            .map(|argument| cmm::ir::Variable::new(argument.name()).into())
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
) -> (Vec<cmm::ir::Instruction>, cmm::ir::Variable) {
    let (instructions, variable) = expressions::compile(definition.body());

    (
        definition
            .environment()
            .iter()
            .enumerate()
            .map(|(index, free_variable)| {
                cmm::ir::Assignment::new(
                    free_variable.name(),
                    cmm::ir::Load::new(cmm::ir::AddressCalculation::new(
                        cmm::ir::Bitcast::new(
                            compile_environment_pointer(),
                            cmm::types::Pointer::new(types::compile_environment(definition)),
                        ),
                        vec![
                            cmm::ir::Primitive::PointerInteger(0).into(),
                            cmm::ir::Primitive::PointerInteger(index as u64).into(),
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

fn compile_normal_entry(definition: &ssf::ir::Definition) -> cmm::ir::FunctionDefinition {
    cmm::ir::FunctionDefinition::new(
        generate_normal_entry_name(definition.name()),
        compile_arguments(definition),
        compile_normal_body(definition),
        types::compile(definition.result_type()),
    )
}

fn compile_locked_entry(definition: &ssf::ir::Definition) -> cmm::ir::FunctionDefinition {
    let entry_function_name = generate_locked_entry_name(definition.name());

    cmm::ir::FunctionDefinition::new(
        &entry_function_name,
        compile_arguments(definition),
        vec![cmm::ir::If::new(
            cmm::ir::PrimitiveOperation::new(
                cmm::ir::PrimitiveOperator::Equal,
                cmm::ir::AtomicLoad::new(compile_entry_pointer(
                    &types::compile_entry_function_from_definition(definition),
                )),
                cmm::ir::Variable::new(&entry_function_name),
            ),
            vec![cmm::ir::Instruction::Unreachable],
            compile_normal_body(definition),
        )
        .into()],
        types::compile(definition.result_type()),
    )
}

fn compile_normal_body(definition: &ssf::ir::Definition) -> Vec<cmm::ir::Instruction> {
    vec![
        cmm::ir::Return::new(cmm::ir::Load::new(cmm::ir::Bitcast::new(
            compile_environment_pointer(),
            cmm::types::Pointer::new(types::compile(definition.result_type())),
        )))
        .into(),
    ]
}

fn compile_entry_pointer(entry_function_type: &cmm::types::Function) -> cmm::ir::Expression {
    cmm::ir::AddressCalculation::new(
        cmm::ir::Bitcast::new(
            compile_environment_pointer(),
            cmm::types::Pointer::new(cmm::types::Pointer::new(entry_function_type.clone())),
        ),
        vec![cmm::ir::Primitive::PointerInteger(-2i64 as u64).into()],
    )
    .into()
}

fn compile_environment_pointer() -> cmm::ir::Variable {
    cmm::ir::Variable::new(ENVIRONMENT_NAME)
}

fn compile_arguments(definition: &ssf::ir::Definition) -> Vec<cmm::ir::Argument> {
    vec![cmm::ir::Argument::new(
        ENVIRONMENT_NAME,
        cmm::types::Pointer::new(types::compile_environment(definition)),
    )]
    .into_iter()
    .chain(
        definition.arguments().iter().map(|argument| {
            cmm::ir::Argument::new(argument.name(), types::compile(argument.type_()))
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
