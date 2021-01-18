use super::closures;
use super::types;
use super::utilities;

// Closures' entry pointers are uncurried.
pub fn compile(
    builder: &fmm::build::BlockBuilder,
    closure_pointer: fmm::build::TypedExpression,
    arguments: &[fmm::build::TypedExpression],
) -> fmm::build::TypedExpression {
    if arguments.is_empty() {
        closure_pointer
    } else if arguments.len() == 1 {
        compile_single_argument_application(&builder, closure_pointer.clone(), arguments[0].clone())
    } else {
        builder.if_(
            builder.comparison_operation(
                fmm::ir::ComparisonOperator::LessThan,
                fmm::ir::Primitive::PointerInteger(arguments.len() as u64),
                closures::compile_load_arity(builder, closure_pointer.clone()),
            ),
            |builder| {
                builder.branch(compile_create_closure(
                    &builder,
                    closure_pointer.clone(),
                    arguments,
                ))
            },
            |builder| {
                builder.branch(compile(
                    &builder,
                    compile_single_argument_application(
                        &builder,
                        closure_pointer.clone(),
                        arguments[0].clone(),
                    ),
                    &arguments[1..],
                ))
            },
        )
    }
}

fn compile_single_argument_application(
    builder: &fmm::build::BlockBuilder,
    closure_pointer: fmm::build::TypedExpression,
    argument: fmm::build::TypedExpression,
) -> fmm::build::TypedExpression {
    if types::get_arity(get_entry_function_type(&closure_pointer)) == 1 {
        compile_single_argument_direct_call(&builder, closure_pointer, argument)
    } else {
        builder.if_(
            builder.comparison_operation(
                fmm::ir::ComparisonOperator::Equal,
                closures::compile_load_arity(builder, closure_pointer.clone()),
                fmm::ir::Primitive::PointerInteger(1),
            ),
            |builder| {
                builder.branch(compile_single_argument_direct_call(
                    &builder,
                    closure_pointer.clone(),
                    argument.clone(),
                ))
            },
            |builder| {
                builder.branch(compile_create_closure(
                    &builder,
                    closure_pointer.clone(),
                    &[argument.clone()],
                ))
            },
        )
    }
}

fn compile_single_argument_direct_call(
    builder: &fmm::build::BlockBuilder,
    closure_pointer: fmm::build::TypedExpression,
    argument: fmm::build::TypedExpression,
) -> fmm::build::TypedExpression {
    let entry_pointer = closures::compile_load_entry_pointer(&builder, closure_pointer.clone());

    builder.call(
        utilities::bitcast(
            &builder,
            entry_pointer.clone(),
            types::compile_curried_entry_function(entry_pointer.type_().to_function().unwrap(), 1),
        ),
        vec![
            closures::compile_environment_pointer(&builder, closure_pointer.clone()),
            argument.clone(),
        ],
    )
}

fn compile_create_closure(
    builder: &fmm::build::BlockBuilder,
    closure_pointer: fmm::build::TypedExpression,
    arguments: &[fmm::build::TypedExpression],
) -> fmm::build::TypedExpression {
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
    );

    let closure = closures::compile_closure_content(
        compile_partially_applied_entry_function(
            builder,
            &target_entry_function_type,
            &closure_pointer.type_(),
            &arguments
                .iter()
                .map(|argument| argument.type_())
                .collect::<Vec<_>>(),
        ),
        vec![closure_pointer]
            .into_iter()
            .chain(arguments.iter().cloned())
            .collect::<Vec<_>>(),
    );
    let closure_pointer = builder.allocate_heap(closure.type_().clone());
    builder.store(closure, closure_pointer.clone());

    utilities::bitcast(
        builder,
        closure_pointer,
        fmm::types::Pointer::new(types::compile_raw_closure(
            target_entry_function_type,
            types::compile_unsized_environment(),
        )),
    )
}

fn compile_partially_applied_entry_function(
    builder: &fmm::build::BlockBuilder,
    entry_function_type: &fmm::types::Function,
    closure_pointer_type: &fmm::types::Type,
    argument_types: &[&fmm::types::Type],
) -> fmm::build::TypedExpression {
    let curried_entry_function_type =
        types::compile_curried_entry_function(&entry_function_type, 1);
    let arguments = curried_entry_function_type
        .arguments()
        .iter()
        .enumerate()
        .map(|(index, type_)| fmm::ir::Argument::new(format!("arg_{}", index), type_.clone()))
        .collect::<Vec<_>>();

    builder.module_builder().define_anonymous_function(
        arguments.clone(),
        |builder| {
            let environment_pointer =
                utilities::variable(arguments[0].name(), arguments[0].type_().clone());
            let new_argument =
                utilities::variable(arguments[0].name(), arguments[0].type_().clone());

            let environment = builder.load(utilities::bitcast(
                &builder,
                environment_pointer,
                fmm::types::Pointer::new(fmm::types::Record::new(
                    vec![closure_pointer_type.clone()]
                        .into_iter()
                        .chain(argument_types.iter().cloned().cloned())
                        .collect(),
                )),
            ));

            builder.return_(compile(
                &builder,
                builder.deconstruct_record(environment.clone(), 0),
                &(0..argument_types.len())
                    .map(|index| builder.deconstruct_record(environment.clone(), index + 1))
                    .chain(vec![new_argument])
                    .collect::<Vec<_>>(),
            ))
        },
        curried_entry_function_type.result().clone(),
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
