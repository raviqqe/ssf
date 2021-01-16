use super::expressions;
use super::types;
use super::utilities;

const ENVIRONMENT_NAME: &str = "_environment";

pub fn compile(definition: &ssf::ir::Definition) -> Vec<fmm::ir::FunctionDefinition> {
    if definition.is_thunk() {
        compile_thunk(definition)
    } else {
        vec![compile_non_thunk(definition)]
    }
}

fn compile_non_thunk(definition: &ssf::ir::Definition) -> fmm::ir::FunctionDefinition {
    let state = fmm::build::BlockState::new();

    fmm::ir::FunctionDefinition::new(
        generate_closure_entry_name(definition.name()),
        compile_arguments(definition),
        state.return_(compile_body(&state, definition)),
        types::compile(definition.result_type()),
    )
}

fn compile_thunk(definition: &ssf::ir::Definition) -> Vec<fmm::ir::FunctionDefinition> {
    vec![
        compile_first_thunk_entry(definition),
        compile_normal_thunk_entry(definition),
        compile_locked_thunk_entry(definition),
    ]
}

fn compile_body(
    state: &fmm::build::BlockState,
    definition: &ssf::ir::Definition,
) -> fmm::build::TypedExpression {
    expressions::compile(
        state,
        definition.body(),
        &definition
            .environment()
            .iter()
            .enumerate()
            .map(|(index, free_variable)| {
                (
                    free_variable.name().into(),
                    state.load(state.record_address(
                        utilities::bitcast(
                            state,
                            compile_environment_pointer(),
                            fmm::types::Pointer::new(types::compile_environment(definition)),
                        ),
                        index,
                    )),
                )
            })
            .collect(),
    )
}

fn compile_first_thunk_entry(definition: &ssf::ir::Definition) -> fmm::ir::FunctionDefinition {
    let entry_function_name = generate_closure_entry_name(definition.name());
    let entry_function_type = types::compile_entry_function_from_definition(definition);
    let arguments = compile_arguments(definition);

    fmm::ir::FunctionDefinition::new(
        &entry_function_name,
        arguments.clone(),
        {
            let state = fmm::build::BlockState::new();

            state.if_(
                state.compare_and_swap(
                    compile_entry_function_pointer_pointer(&state, definition),
                    utilities::variable(&entry_function_name, entry_function_type.clone()),
                    utilities::variable(
                        generate_locked_entry_name(definition.name()),
                        entry_function_type.clone(),
                    ),
                ),
                |state| {
                    let value = compile_body(&state, definition);

                    state.store(
                        value.clone(),
                        utilities::bitcast(
                            &state,
                            compile_environment_pointer(),
                            fmm::types::Pointer::new(types::compile(definition.result_type())),
                        ),
                    );
                    state.atomic_store(
                        utilities::variable(
                            generate_normal_entry_name(definition.name()),
                            entry_function_type.clone(),
                        ),
                        compile_entry_function_pointer_pointer(&state, definition),
                    );

                    state.return_(value)
                },
                |state| {
                    state.return_(
                        state.call(
                            state.atomic_load(compile_entry_function_pointer_pointer(
                                &state, definition,
                            )),
                            arguments
                                .iter()
                                .map(|argument| {
                                    utilities::variable(argument.name(), argument.type_().clone())
                                })
                                .collect(),
                        ),
                    )
                },
            );

            state.unreachable()
        },
        types::compile(definition.result_type()),
    )
}

fn compile_normal_thunk_entry(definition: &ssf::ir::Definition) -> fmm::ir::FunctionDefinition {
    fmm::ir::FunctionDefinition::new(
        generate_normal_entry_name(definition.name()),
        compile_arguments(definition),
        compile_normal_body(&fmm::build::BlockState::new(), definition),
        types::compile(definition.result_type()),
    )
}

fn compile_locked_thunk_entry(definition: &ssf::ir::Definition) -> fmm::ir::FunctionDefinition {
    let entry_function_name = generate_locked_entry_name(definition.name());

    fmm::ir::FunctionDefinition::new(
        &entry_function_name,
        compile_arguments(definition),
        {
            let state = fmm::build::BlockState::new();

            state.if_(
                state.comparison_operation(
                    fmm::ir::ComparisonOperator::Equal,
                    utilities::bitcast(
                        &state,
                        state.atomic_load(compile_entry_function_pointer_pointer(
                            &state, definition,
                        )),
                        fmm::types::Primitive::PointerInteger,
                    ),
                    utilities::bitcast(
                        &state,
                        utilities::variable(
                            &entry_function_name,
                            types::compile_entry_function_from_definition(definition),
                        ),
                        fmm::types::Primitive::PointerInteger,
                    ),
                ),
                // TODO Return to handle thunk locks asynchronously.
                |state| state.unreachable(),
                |state| compile_normal_body(&state, definition),
            );

            state.unreachable()
        },
        types::compile(definition.result_type()),
    )
}

fn compile_normal_body(
    state: &fmm::build::BlockState,
    definition: &ssf::ir::Definition,
) -> fmm::ir::Block {
    state.return_(state.load(utilities::bitcast(
        &state,
        compile_environment_pointer(),
        fmm::types::Pointer::new(types::compile(definition.result_type())),
    )))
}

fn compile_entry_function_pointer_pointer(
    state: &fmm::build::BlockState,
    definition: &ssf::ir::Definition,
) -> fmm::build::TypedExpression {
    // TODO Calculate entry function pointer properly.
    // The offset should be calculated by allocating a record of
    // { pointer, { pointer, arity, environment } }.
    state.pointer_address(
        utilities::bitcast(
            state,
            compile_environment_pointer(),
            fmm::types::Pointer::new(types::compile_entry_function_from_definition(definition)),
        ),
        fmm::ir::Primitive::PointerInteger(-2i64 as u64),
    )
}

fn compile_arguments(definition: &ssf::ir::Definition) -> Vec<fmm::ir::Argument> {
    vec![fmm::ir::Argument::new(
        ENVIRONMENT_NAME,
        types::compile_unsized_environment(),
    )]
    .into_iter()
    .chain(
        definition.arguments().iter().map(|argument| {
            fmm::ir::Argument::new(argument.name(), types::compile(argument.type_()))
        }),
    )
    .collect()
}

fn compile_environment_pointer() -> fmm::build::TypedExpression {
    fmm::build::TypedExpression::new(
        fmm::ir::Variable::new(ENVIRONMENT_NAME),
        types::compile_unsized_environment(),
    )
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
