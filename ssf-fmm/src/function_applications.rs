use super::closures;
use super::expressions;
use super::types;

pub fn compile(
    module_builder: &fmm::build::ModuleBuilder,
    instruction_builder: &fmm::build::InstructionBuilder,
    closure_pointer: fmm::build::TypedExpression,
    arguments: &[fmm::build::TypedExpression],
) -> Result<fmm::build::TypedExpression, fmm::build::BuildError> {
    compile_with_min_arity(
        module_builder,
        instruction_builder,
        closure_pointer,
        arguments,
        1,
    )
}

fn compile_with_min_arity(
    module_builder: &fmm::build::ModuleBuilder,
    instruction_builder: &fmm::build::InstructionBuilder,
    closure_pointer: fmm::build::TypedExpression,
    arguments: &[fmm::build::TypedExpression],
    min_arity: usize,
) -> Result<fmm::build::TypedExpression, fmm::build::BuildError> {
    Ok(if arguments.is_empty() {
        closure_pointer
    } else if arguments.len() < min_arity {
        compile_create_closure(
            module_builder,
            instruction_builder,
            closure_pointer,
            arguments,
        )?
    } else if types::get_arity(get_entry_function_type(&closure_pointer)) == min_arity {
        compile_direct_call(instruction_builder, closure_pointer, arguments)?
    } else {
        instruction_builder.if_(
            fmm::build::comparison_operation(
                fmm::ir::ComparisonOperator::Equal,
                closures::compile_load_arity(instruction_builder, closure_pointer.clone())?,
                expressions::compile_arity(min_arity),
            )?,
            |instruction_builder| {
                Ok(instruction_builder.branch(compile(
                    module_builder,
                    &instruction_builder,
                    compile_direct_call(
                        &instruction_builder,
                        closure_pointer.clone(),
                        &arguments[..min_arity],
                    )?,
                    &arguments[min_arity..],
                )?))
            },
            |instruction_builder| {
                Ok(instruction_builder.branch(compile_with_min_arity(
                    module_builder,
                    &instruction_builder,
                    closure_pointer.clone(),
                    arguments,
                    min_arity + 1,
                )?))
            },
        )?
    })
}

fn compile_direct_call(
    instruction_builder: &fmm::build::InstructionBuilder,
    closure_pointer: fmm::build::TypedExpression,
    arguments: &[fmm::build::TypedExpression],
) -> Result<fmm::build::TypedExpression, fmm::build::BuildError> {
    instruction_builder.call(
        fmm::build::bit_cast(
            types::compile_curried_entry_function(
                get_entry_function_type(&closure_pointer),
                arguments.len(),
            ),
            closures::compile_load_entry_pointer(&instruction_builder, closure_pointer.clone())?,
        ),
        vec![closures::compile_environment_pointer(
            &instruction_builder,
            closure_pointer,
        )?]
        .into_iter()
        .chain(arguments.iter().cloned())
        .collect(),
    )
}

fn compile_create_closure(
    module_builder: &fmm::build::ModuleBuilder,
    instruction_builder: &fmm::build::InstructionBuilder,
    closure_pointer: fmm::build::TypedExpression,
    arguments: &[fmm::build::TypedExpression],
) -> Result<fmm::build::TypedExpression, fmm::build::BuildError> {
    let entry_function_type = get_entry_function_type(&closure_pointer);

    let target_entry_function_type = fmm::types::Function::new(
        entry_function_type.arguments()[..types::FUNCTION_ARGUMENT_OFFSET]
            .iter()
            .cloned()
            .chain(
                entry_function_type.arguments()
                    [arguments.len() + types::FUNCTION_ARGUMENT_OFFSET..]
                    .iter()
                    .cloned(),
            )
            .collect(),
        entry_function_type.result().clone(),
        fmm::types::CallingConvention::Source,
    );

    let closure = closures::compile_closure_content(
        compile_partially_applied_entry_function(
            module_builder,
            &target_entry_function_type,
            &closure_pointer.type_(),
            &arguments
                .iter()
                .map(|argument| argument.type_())
                .collect::<Vec<_>>(),
        )?,
        vec![closure_pointer]
            .into_iter()
            .chain(arguments.iter().cloned())
            .collect::<Vec<_>>(),
    );
    let closure_pointer = instruction_builder.allocate_heap(closure.type_().clone());
    instruction_builder.store(closure, closure_pointer.clone());

    Ok(fmm::build::bit_cast(
        fmm::types::Pointer::new(types::compile_raw_closure(
            target_entry_function_type,
            types::compile_unsized_environment(),
        )),
        closure_pointer,
    )
    .into())
}

fn compile_partially_applied_entry_function(
    module_builder: &fmm::build::ModuleBuilder,
    entry_function_type: &fmm::types::Function,
    closure_pointer_type: &fmm::types::Type,
    argument_types: &[&fmm::types::Type],
) -> Result<fmm::build::TypedExpression, fmm::build::BuildError> {
    let curried_entry_function_type =
        types::compile_curried_entry_function(&entry_function_type, 1);
    let arguments = curried_entry_function_type
        .arguments()
        .iter()
        .enumerate()
        .map(|(index, type_)| fmm::ir::Argument::new(format!("arg_{}", index), type_.clone()))
        .collect::<Vec<_>>();

    module_builder.define_anonymous_function(
        arguments.clone(),
        |instruction_builder| {
            let environment = instruction_builder.load(fmm::build::bit_cast(
                fmm::types::Pointer::new(fmm::types::Record::new(
                    vec![closure_pointer_type.clone()]
                        .into_iter()
                        .chain(argument_types.iter().cloned().cloned())
                        .collect(),
                )),
                fmm::build::variable(arguments[0].name(), arguments[0].type_().clone()),
            ))?;
            let closure_pointer = instruction_builder.deconstruct_record(environment.clone(), 0)?;
            let arguments = (0..argument_types.len())
                .map(|index| instruction_builder.deconstruct_record(environment.clone(), index + 1))
                .chain(vec![Ok(fmm::build::variable(
                    arguments[1].name(),
                    arguments[1].type_().clone(),
                ))])
                .collect::<Result<Vec<_>, _>>()?;

            Ok(instruction_builder.return_(
                if types::get_arity(get_entry_function_type(&closure_pointer)) == arguments.len() {
                    compile_direct_call(&instruction_builder, closure_pointer, &arguments)?
                } else {
                    instruction_builder.if_(
                        fmm::build::comparison_operation(
                            fmm::ir::ComparisonOperator::Equal,
                            closures::compile_load_arity(
                                &instruction_builder,
                                closure_pointer.clone(),
                            )?,
                            expressions::compile_arity(arguments.len()),
                        )?,
                        |instruction_builder| {
                            Ok(instruction_builder.branch(compile_direct_call(
                                &instruction_builder,
                                closure_pointer.clone(),
                                &arguments,
                            )?))
                        },
                        |instruction_builder| {
                            Ok(instruction_builder.branch(compile_create_closure(
                                module_builder,
                                &instruction_builder,
                                closure_pointer.clone(),
                                &arguments,
                            )?))
                        },
                    )?
                },
            ))
        },
        curried_entry_function_type.result().clone(),
        fmm::types::CallingConvention::Source,
    )
}

fn get_entry_function_type(closure_pointer: &fmm::build::TypedExpression) -> &fmm::types::Function {
    closure_pointer
        .type_()
        .to_pointer()
        .unwrap()
        .element()
        .to_record()
        .unwrap()
        .elements()[0]
        .to_function()
        .unwrap()
}
