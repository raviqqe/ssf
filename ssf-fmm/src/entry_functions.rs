use crate::expressions;
use crate::types;
use crate::utilities;
use crate::variable_builder::VariableBuilder;
use std::collections::HashMap;

const ENVIRONMENT_NAME: &str = "_env";

pub fn compile(
    module_builder: &fmm::build::ModuleBuilder,
    definition: &ssf::ir::Definition,
    variables: &HashMap<String, VariableBuilder>,
) -> fmm::build::TypedExpression {
    if definition.is_thunk() {
        compile_thunk(module_builder, definition, variables)
    } else {
        compile_non_thunk(module_builder, definition, variables)
    }
}

fn compile_non_thunk(
    module_builder: &fmm::build::ModuleBuilder,
    definition: &ssf::ir::Definition,
    variables: &HashMap<String, VariableBuilder>,
) -> fmm::build::TypedExpression {
    module_builder.define_anonymous_function(
        compile_arguments(definition),
        |instruction_builder| {
            instruction_builder.return_(compile_body(
                module_builder,
                &instruction_builder,
                definition,
                variables,
            ))
        },
        types::compile(definition.result_type()),
        fmm::types::CallingConvention::Source,
    )
}

fn compile_thunk(
    module_builder: &fmm::build::ModuleBuilder,
    definition: &ssf::ir::Definition,
    variables: &HashMap<String, VariableBuilder>,
) -> fmm::build::TypedExpression {
    compile_first_thunk_entry(
        module_builder,
        definition,
        compile_normal_thunk_entry(module_builder, definition),
        compile_locked_thunk_entry(module_builder, definition),
        variables,
    )
}

fn compile_body(
    module_builder: &fmm::build::ModuleBuilder,
    instruction_builder: &fmm::build::InstructionBuilder,
    definition: &ssf::ir::Definition,
    variables: &HashMap<String, VariableBuilder>,
) -> fmm::build::TypedExpression {
    expressions::compile(
        module_builder,
        instruction_builder,
        definition.body(),
        &variables
            .clone()
            .into_iter()
            .chain(
                definition
                    .environment()
                    .iter()
                    .enumerate()
                    .map(|(index, free_variable)| {
                        (
                            free_variable.name().into(),
                            instruction_builder
                                .load(instruction_builder.record_address(
                                    utilities::bitcast(
                                        instruction_builder,
                                        compile_environment_pointer(),
                                        fmm::types::Pointer::new(types::compile_environment(
                                            definition,
                                        )),
                                    ),
                                    index,
                                ))
                                .into(),
                        )
                    }),
            )
            .chain(definition.arguments().iter().map(|argument| {
                (
                    argument.name().into(),
                    fmm::build::variable(argument.name(), types::compile(argument.type_())).into(),
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
    variables: &HashMap<String, VariableBuilder>,
) -> fmm::build::TypedExpression {
    let entry_function_name = module_builder.generate_name();
    let entry_function_type = types::compile_entry_function_from_definition(definition);
    let arguments = compile_arguments(definition);

    module_builder.define_function(
        &entry_function_name,
        arguments.clone(),
        |instruction_builder| {
            instruction_builder.if_(
                instruction_builder.compare_and_swap(
                    compile_entry_function_pointer_pointer(&instruction_builder, definition),
                    fmm::build::variable(&entry_function_name, entry_function_type.clone()),
                    lock_entry_function.clone(),
                ),
                |instruction_builder| {
                    let value =
                        compile_body(module_builder, &instruction_builder, definition, variables);

                    instruction_builder.store(
                        value.clone(),
                        utilities::bitcast(
                            &instruction_builder,
                            compile_environment_pointer(),
                            fmm::types::Pointer::new(types::compile(definition.result_type())),
                        ),
                    );
                    instruction_builder.atomic_store(
                        normal_entry_function.clone(),
                        compile_entry_function_pointer_pointer(&instruction_builder, definition),
                    );

                    instruction_builder.return_(value)
                },
                |instruction_builder| {
                    instruction_builder.return_(
                        instruction_builder.call(
                            instruction_builder.atomic_load(
                                compile_entry_function_pointer_pointer(
                                    &instruction_builder,
                                    definition,
                                ),
                            ),
                            arguments
                                .iter()
                                .map(|argument| {
                                    fmm::build::variable(argument.name(), argument.type_().clone())
                                })
                                .collect(),
                        ),
                    )
                },
            );

            instruction_builder.unreachable()
        },
        types::compile(definition.result_type()),
        fmm::types::CallingConvention::Source,
        false,
    )
}

fn compile_normal_thunk_entry(
    module_builder: &fmm::build::ModuleBuilder,
    definition: &ssf::ir::Definition,
) -> fmm::build::TypedExpression {
    module_builder.define_anonymous_function(
        compile_arguments(definition),
        |instruction_builder| compile_normal_body(&instruction_builder, definition),
        types::compile(definition.result_type()),
        fmm::types::CallingConvention::Source,
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
        |instruction_builder| {
            instruction_builder.if_(
                instruction_builder.comparison_operation(
                    fmm::ir::ComparisonOperator::Equal,
                    utilities::bitcast(
                        &instruction_builder,
                        instruction_builder.atomic_load(compile_entry_function_pointer_pointer(
                            &instruction_builder,
                            definition,
                        )),
                        fmm::types::Primitive::PointerInteger,
                    ),
                    utilities::bitcast(
                        &instruction_builder,
                        fmm::build::variable(
                            &entry_function_name,
                            types::compile_entry_function_from_definition(definition),
                        ),
                        fmm::types::Primitive::PointerInteger,
                    ),
                ),
                // TODO Return to handle thunk locks asynchronously.
                |instruction_builder| instruction_builder.unreachable(),
                |instruction_builder| compile_normal_body(&instruction_builder, definition),
            );

            instruction_builder.unreachable()
        },
        types::compile(definition.result_type()),
        fmm::types::CallingConvention::Source,
        false,
    )
}

fn compile_normal_body(
    instruction_builder: &fmm::build::InstructionBuilder,
    definition: &ssf::ir::Definition,
) -> fmm::ir::Block {
    instruction_builder.return_(instruction_builder.load(utilities::bitcast(
        &instruction_builder,
        compile_environment_pointer(),
        fmm::types::Pointer::new(types::compile(definition.result_type())),
    )))
}

fn compile_entry_function_pointer_pointer(
    instruction_builder: &fmm::build::InstructionBuilder,
    definition: &ssf::ir::Definition,
) -> fmm::build::TypedExpression {
    // TODO Calculate entry function pointer properly.
    // The offset should be calculated by allocating a record of
    // { pointer, { pointer, arity, environment } }.
    instruction_builder.pointer_address(
        utilities::bitcast(
            instruction_builder,
            compile_environment_pointer(),
            fmm::types::Pointer::new(types::compile_entry_function_from_definition(definition)),
        ),
        fmm::ir::Primitive::PointerInteger(-2),
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
    fmm::build::variable(
        ENVIRONMENT_NAME,
        fmm::types::Pointer::new(types::compile_unsized_environment()),
    )
}
