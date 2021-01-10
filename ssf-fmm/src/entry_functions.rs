use super::expressions;
use super::types;
use super::utilities::*;
use fmm::build::*;

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
        return_(compile_body(definition)),
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

fn compile_body(definition: &ssf::ir::Definition) -> BuildContext {
    expressions::compile(
        definition.body(),
        &definition
            .environment()
            .iter()
            .enumerate()
            .map(|(index, free_variable)| {
                (
                    free_variable.name().into(),
                    load(record_address(
                        bitcast(
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
        unreachable(if_(
            compare_and_swap(
                compile_entry_function_pointer_pointer(definition),
                variable(&entry_function_name, entry_function_type.clone()),
                variable(
                    generate_locked_entry_name(definition.name()),
                    entry_function_type.clone(),
                ),
            ),
            return_(side_effect(compile_body(definition), |context| {
                store(
                    context,
                    bitcast(
                        compile_environment_pointer(),
                        fmm::types::Pointer::new(types::compile(definition.result_type())),
                    ),
                )
                .into_iter()
                .chain(atomic_store(
                    variable(
                        generate_normal_entry_name(definition.name()),
                        entry_function_type.clone(),
                    ),
                    compile_entry_function_pointer_pointer(definition),
                ))
            })),
            return_(call(
                atomic_load(compile_entry_function_pointer_pointer(definition)),
                arguments
                    .iter()
                    .map(|argument| variable(argument.name(), argument.type_().clone())),
            )),
        )),
        types::compile(definition.result_type()),
    )
}

fn compile_normal_thunk_entry(definition: &ssf::ir::Definition) -> fmm::ir::FunctionDefinition {
    fmm::ir::FunctionDefinition::new(
        generate_normal_entry_name(definition.name()),
        compile_arguments(definition),
        compile_normal_body(definition),
        types::compile(definition.result_type()),
    )
}

fn compile_locked_thunk_entry(definition: &ssf::ir::Definition) -> fmm::ir::FunctionDefinition {
    let entry_function_name = generate_locked_entry_name(definition.name());

    fmm::ir::FunctionDefinition::new(
        &entry_function_name,
        compile_arguments(definition),
        unreachable(if_(
            comparison_operation(
                fmm::ir::ComparisonOperator::Equal,
                bitcast(
                    atomic_load(compile_entry_function_pointer_pointer(definition)),
                    fmm::types::Primitive::PointerInteger,
                ),
                bitcast(
                    variable(
                        &entry_function_name,
                        types::compile_entry_function_from_definition(definition),
                    ),
                    fmm::types::Primitive::PointerInteger,
                ),
            ),
            // TODO Return to handle thunk locks asynchronously.
            fmm::ir::Block::new(vec![], fmm::ir::TerminalInstruction::Unreachable),
            compile_normal_body(definition),
        )),
        types::compile(definition.result_type()),
    )
}

fn compile_normal_body(definition: &ssf::ir::Definition) -> fmm::ir::Block {
    return_(load(bitcast(
        compile_environment_pointer(),
        fmm::types::Pointer::new(types::compile(definition.result_type())),
    )))
}

fn compile_entry_function_pointer_pointer(definition: &ssf::ir::Definition) -> BuildContext {
    // TODO Calculate entry function pointer properly.
    // The offset should be calculated by allocating a record of
    // { pointer, { pointer, arity, environment } }.
    pointer_address(
        bitcast(
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

fn compile_environment_pointer() -> BuildContext {
    variable(ENVIRONMENT_NAME, types::compile_unsized_environment())
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
