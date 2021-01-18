use super::expressions;
use super::types;
use super::utilities;

const ENVIRONMENT_NAME: &str = "_env";

pub fn compile(
    module_builder: &fmm::build::ModuleBuilder,
    definition: &ssf::ir::Definition,
) -> fmm::build::TypedExpression {
    if definition.is_thunk() {
        compile_thunk(module_builder, definition)
    } else {
        compile_non_thunk(module_builder, definition)
    }
}

fn compile_non_thunk(
    module_builder: &fmm::build::ModuleBuilder,
    definition: &ssf::ir::Definition,
) -> fmm::build::TypedExpression {
    module_builder.define_anonymous_function(
        compile_arguments(definition),
        |builder| builder.return_(compile_body(&builder, definition)),
        types::compile(definition.result_type()),
    )
}

fn compile_thunk(
    module_builder: &fmm::build::ModuleBuilder,
    definition: &ssf::ir::Definition,
) -> fmm::build::TypedExpression {
    compile_first_thunk_entry(
        module_builder,
        definition,
        compile_normal_thunk_entry(module_builder, definition),
        compile_locked_thunk_entry(module_builder, definition),
    )
}

fn compile_body(
    builder: &fmm::build::BlockBuilder,
    definition: &ssf::ir::Definition,
) -> fmm::build::TypedExpression {
    expressions::compile(
        builder,
        definition.body(),
        &definition
            .environment()
            .iter()
            .enumerate()
            .map(|(index, free_variable)| {
                (
                    free_variable.name().into(),
                    builder.load(builder.record_address(
                        utilities::bitcast(
                            builder,
                            compile_environment_pointer(),
                            fmm::types::Pointer::new(types::compile_environment(definition)),
                        ),
                        index,
                    )),
                )
            })
            .chain(definition.arguments().iter().map(|argument| {
                (
                    argument.name().into(),
                    utilities::variable(argument.name(), types::compile(argument.type_())),
                )
            }))
            .collect(),
    )
}

fn compile_first_thunk_entry(
    module_builder: &fmm::build::ModuleBuilder,
    definition: &ssf::ir::Definition,
    normal_entry_function: fmm::build::TypedExpression,
    lock_entry_function: fmm::build::TypedExpression,
) -> fmm::build::TypedExpression {
    let entry_function_name = module_builder.generate_name();
    let entry_function_type = types::compile_entry_function_from_definition(definition);
    let arguments = compile_arguments(definition);

    module_builder.define_function(
        &entry_function_name,
        arguments.clone(),
        |builder| {
            builder.if_(
                builder.compare_and_swap(
                    compile_entry_function_pointer_pointer(&builder, definition),
                    utilities::variable(&entry_function_name, entry_function_type.clone()),
                    lock_entry_function.clone(),
                ),
                |builder| {
                    let value = compile_body(&builder, definition);

                    builder.store(
                        value.clone(),
                        utilities::bitcast(
                            &builder,
                            compile_environment_pointer(),
                            fmm::types::Pointer::new(types::compile(definition.result_type())),
                        ),
                    );
                    builder.atomic_store(
                        normal_entry_function.clone(),
                        compile_entry_function_pointer_pointer(&builder, definition),
                    );

                    builder.return_(value)
                },
                |builder| {
                    builder.return_(
                        builder.call(
                            builder.atomic_load(compile_entry_function_pointer_pointer(
                                &builder, definition,
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

            builder.unreachable()
        },
        types::compile(definition.result_type()),
        false,
    )
}

fn compile_normal_thunk_entry(
    module_builder: &fmm::build::ModuleBuilder,
    definition: &ssf::ir::Definition,
) -> fmm::build::TypedExpression {
    module_builder.define_anonymous_function(
        compile_arguments(definition),
        |builder| compile_normal_body(&builder, definition),
        types::compile(definition.result_type()),
    )
}

fn compile_locked_thunk_entry(
    module_builder: &fmm::build::ModuleBuilder,
    definition: &ssf::ir::Definition,
) -> fmm::build::TypedExpression {
    let entry_function_name = module_builder.generate_name();

    module_builder.define_function(
        &entry_function_name,
        compile_arguments(definition),
        |builder| {
            builder.if_(
                builder.comparison_operation(
                    fmm::ir::ComparisonOperator::Equal,
                    utilities::bitcast(
                        &builder,
                        builder.atomic_load(compile_entry_function_pointer_pointer(
                            &builder, definition,
                        )),
                        fmm::types::Primitive::PointerInteger,
                    ),
                    utilities::bitcast(
                        &builder,
                        utilities::variable(
                            &entry_function_name,
                            types::compile_entry_function_from_definition(definition),
                        ),
                        fmm::types::Primitive::PointerInteger,
                    ),
                ),
                // TODO Return to handle thunk locks asynchronously.
                |builder| builder.unreachable(),
                |builder| compile_normal_body(&builder, definition),
            );

            builder.unreachable()
        },
        types::compile(definition.result_type()),
        false,
    )
}

fn compile_normal_body(
    builder: &fmm::build::BlockBuilder,
    definition: &ssf::ir::Definition,
) -> fmm::ir::Block {
    builder.return_(builder.load(utilities::bitcast(
        &builder,
        compile_environment_pointer(),
        fmm::types::Pointer::new(types::compile(definition.result_type())),
    )))
}

fn compile_entry_function_pointer_pointer(
    builder: &fmm::build::BlockBuilder,
    definition: &ssf::ir::Definition,
) -> fmm::build::TypedExpression {
    // TODO Calculate entry function pointer properly.
    // The offset should be calculated by allocating a record of
    // { pointer, { pointer, arity, environment } }.
    builder.pointer_address(
        utilities::bitcast(
            builder,
            compile_environment_pointer(),
            fmm::types::Pointer::new(types::compile_entry_function_from_definition(definition)),
        ),
        fmm::ir::Primitive::PointerInteger(-2i64 as u64),
    )
}

fn compile_arguments(definition: &ssf::ir::Definition) -> Vec<fmm::ir::Argument> {
    vec![fmm::ir::Argument::new(
        ENVIRONMENT_NAME,
        fmm::types::Pointer::new(types::compile_unsized_environment()),
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
