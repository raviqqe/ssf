use super::utilities;
use crate::closures;
use crate::entry_functions;
use crate::function_applications;
use crate::types;
use std::collections::HashMap;

pub fn compile_arity(arity: u64) -> fmm::ir::Primitive {
    fmm::ir::Primitive::PointerInteger(arity)
}

pub fn compile(
    state: &fmm::build::BlockState,
    expression: &ssf::ir::Expression,
    variables: &HashMap<String, fmm::build::TypedExpression>,
) -> fmm::build::TypedExpression {
    match expression {
        ssf::ir::Expression::Bitcast(bitcast) => utilities::bitcast(
            state,
            compile(state, bitcast.expression(), variables),
            types::compile(bitcast.type_()),
        ),
        ssf::ir::Expression::Case(case) => compile_case(state, case, variables),
        ssf::ir::Expression::ConstructorApplication(constructor_application) => {
            let constructor = constructor_application.constructor();
            let algebraic_type = constructor.algebraic_type();
            let constructor_type =
                algebraic_type.unfold().constructors()[&constructor.tag()].clone();

            utilities::record(
                if algebraic_type.is_singleton() {
                    None
                } else {
                    Some(fmm::ir::Primitive::PointerInteger(constructor.tag()).into())
                }
                .into_iter()
                .chain(if constructor_type.is_enum() {
                    None
                } else {
                    let record_type = types::compile_unboxed_constructor(&constructor_type);
                    let payload = fmm::ir::Record::new(
                        record_type.clone(),
                        constructor_application
                            .arguments()
                            .iter()
                            .map(|argument| {
                                compile(state, argument, variables).expression().clone()
                            })
                            .collect(),
                    );

                    Some(
                        fmm::ir::Union::new(
                            types::compile_constructor_union(algebraic_type),
                            types::get_constructor_union_index(algebraic_type, constructor.tag()),
                            if constructor_type.is_boxed() {
                                let pointer = state.allocate_heap(record_type);
                                state.store(payload, pointer.clone());
                                pointer.expression().clone()
                            } else {
                                payload.into()
                            },
                        )
                        .into(),
                    )
                })
                .collect(),
            )
            .into()
        }
        ssf::ir::Expression::FunctionApplication(function_application) => {
            function_applications::compile(
                state,
                compile(state, function_application.first_function(), variables),
                function_application
                    .arguments()
                    .into_iter()
                    .map(|argument| compile(state, argument, variables))
                    .collect(),
            )
        }
        ssf::ir::Expression::Let(let_) => compile_let(state, let_, variables),
        ssf::ir::Expression::LetRecursive(let_recursive) => {
            compile_let_recursive(state, let_recursive, variables)
        }
        ssf::ir::Expression::Primitive(primitive) => compile_primitive(primitive).into(),
        ssf::ir::Expression::PrimitiveOperation(operation) => {
            compile_primitive_operation(state, operation, variables)
        }
        ssf::ir::Expression::Variable(variable) => variables.get(variable.name()).unwrap().clone(),
    }
}

fn compile_case(
    state: &fmm::build::BlockState,
    case: &ssf::ir::Case,
    variables: &HashMap<String, fmm::build::TypedExpression>,
) -> fmm::build::TypedExpression {
    let compile = |expression| compile(state, expression, variables);

    match case {
        ssf::ir::Case::Algebraic(case) => {
            let argument = compile(case.argument());

            compile_algebraic_alternatives(
                state,
                if case
                    .alternatives()
                    .get(0)
                    .map(|alternative| alternative.constructor().algebraic_type().is_singleton())
                    .unwrap_or(true)
                {
                    fmm::ir::Primitive::PointerInteger(0).into()
                } else {
                    state.deconstruct_record(argument.clone(), 0)
                },
                argument,
                case.alternatives(),
                case.default_alternative(),
                variables,
            )
            .unwrap()
        }
        ssf::ir::Case::Primitive(case) => compile_primitive_case(state, case, variables),
    }
}

fn compile_algebraic_alternatives(
    state: &fmm::build::BlockState,
    tag: fmm::build::TypedExpression,
    argument: fmm::build::TypedExpression,
    alternatives: &[ssf::ir::AlgebraicAlternative],
    default_alternative: Option<&ssf::ir::Expression>,
    variables: &HashMap<String, fmm::build::TypedExpression>,
) -> Option<fmm::build::TypedExpression> {
    Some(match alternatives {
        [] => compile(state, default_alternative?, variables),
        [alternative, ..] => {
            let constructor = alternative.constructor();

            state.if_(
                state.comparison_operation(
                    fmm::ir::ComparisonOperator::Equal,
                    tag.clone(),
                    fmm::ir::Primitive::PointerInteger(constructor.tag()),
                ),
                {
                    let state = fmm::build::BlockState::new();

                    state.branch(compile(
                        &state,
                        alternative.expression(),
                        &if constructor.constructor_type().is_enum() {
                            variables.clone()
                        } else {
                            let mut payload = state.deconstruct_record(
                                argument.clone(),
                                if constructor.algebraic_type().is_singleton() {
                                    0
                                } else {
                                    1
                                },
                            );

                            if constructor.constructor_type().is_boxed() {
                                payload = state.load(payload);
                            }

                            variables
                                .clone()
                                .into_iter()
                                .chain(alternative.element_names().iter().enumerate().map(
                                    |(index, name)| {
                                        (
                                            name.into(),
                                            state.deconstruct_record(payload.clone(), index),
                                        )
                                    },
                                ))
                                .collect()
                        },
                    ))
                },
                {
                    let state = fmm::build::BlockState::new();

                    if let Some(expression) = compile_algebraic_alternatives(
                        &state,
                        tag,
                        argument,
                        &alternatives[1..],
                        default_alternative,
                        variables,
                    ) {
                        state.branch(expression)
                    } else {
                        state.unreachable()
                    }
                },
            )
        }
    })
}

fn compile_primitive_case(
    state: &fmm::build::BlockState,
    case: &ssf::ir::PrimitiveCase,
    variables: &HashMap<String, fmm::build::TypedExpression>,
) -> fmm::build::TypedExpression {
    let argument = compile(state, case.argument(), variables);

    compile_primitive_alternatives(
        state,
        argument,
        case.alternatives(),
        case.default_alternative(),
        variables,
    )
    .unwrap()
}

fn compile_primitive_alternatives(
    state: &fmm::build::BlockState,
    argument: fmm::build::TypedExpression,
    alternatives: &[ssf::ir::PrimitiveAlternative],
    default_alternative: Option<&ssf::ir::Expression>,
    variables: &HashMap<String, fmm::build::TypedExpression>,
) -> Option<fmm::build::TypedExpression> {
    match alternatives {
        [] => default_alternative.map(|expression| compile(state, expression, variables)),
        [alternative, ..] => Some(state.if_(
            state.comparison_operation(
                fmm::ir::ComparisonOperator::Equal,
                argument.clone(),
                compile_primitive(alternative.primitive()),
            ),
            {
                let state = fmm::build::BlockState::new();

                state.branch(compile(&state, alternative.expression(), variables))
            },
            {
                let state = fmm::build::BlockState::new();

                if let Some(expression) = compile_primitive_alternatives(
                    &state,
                    argument,
                    &alternatives[1..],
                    default_alternative,
                    variables,
                ) {
                    state.branch(expression)
                } else {
                    state.unreachable()
                }
            },
        )),
    }
}

fn compile_let(
    state: &fmm::build::BlockState,
    let_: &ssf::ir::Let,
    variables: &HashMap<String, fmm::build::TypedExpression>,
) -> fmm::build::TypedExpression {
    compile(
        state,
        let_.expression(),
        &variables
            .clone()
            .drain()
            .chain(vec![(
                let_.name().into(),
                compile(state, let_.bound_expression(), variables),
            )])
            .collect(),
    )
}

fn compile_let_recursive(
    state: &fmm::build::BlockState,
    let_: &ssf::ir::LetRecursive,
    variables: &HashMap<String, fmm::build::TypedExpression>,
) -> fmm::build::TypedExpression {
    let mut variables = variables.clone();

    for definition in let_.definitions() {
        variables.insert(
            definition.name().into(),
            state.allocate_heap(types::compile_sized_closure(definition)),
        );
    }

    for definition in let_.definitions() {
        state.store(
            closures::compile_closure_content(
                utilities::variable(
                    entry_functions::generate_closure_entry_name(definition.name()),
                    types::compile_entry_function_from_definition(definition),
                ),
                definition
                    .environment()
                    .iter()
                    .map(|free_variable| variables[free_variable.name()].clone())
                    .collect::<Vec<_>>(),
            ),
            variables[definition.name()].clone(),
        );
    }

    compile(state, let_.expression(), &variables)
}

fn compile_primitive_operation(
    state: &fmm::build::BlockState,
    operation: &ssf::ir::PrimitiveOperation,
    variables: &HashMap<String, fmm::build::TypedExpression>,
) -> fmm::build::TypedExpression {
    let lhs = compile(state, operation.lhs(), variables);
    let rhs = compile(state, operation.rhs(), variables);

    match operation.operator() {
        ssf::ir::PrimitiveOperator::Add => {
            state.arithmetic_operation(fmm::ir::ArithmeticOperator::Add, lhs, rhs)
        }
        ssf::ir::PrimitiveOperator::Subtract => {
            state.arithmetic_operation(fmm::ir::ArithmeticOperator::Subtract, lhs, rhs)
        }
        ssf::ir::PrimitiveOperator::Multiply => {
            state.arithmetic_operation(fmm::ir::ArithmeticOperator::Multiply, lhs, rhs)
        }
        ssf::ir::PrimitiveOperator::Divide => {
            state.arithmetic_operation(fmm::ir::ArithmeticOperator::Divide, lhs, rhs)
        }
        ssf::ir::PrimitiveOperator::Equal => {
            state.comparison_operation(fmm::ir::ComparisonOperator::Equal, lhs, rhs)
        }
        ssf::ir::PrimitiveOperator::NotEqual => {
            state.comparison_operation(fmm::ir::ComparisonOperator::NotEqual, lhs, rhs)
        }
        ssf::ir::PrimitiveOperator::LessThan => {
            state.comparison_operation(fmm::ir::ComparisonOperator::LessThan, lhs, rhs)
        }
        ssf::ir::PrimitiveOperator::LessThanOrEqual => {
            state.comparison_operation(fmm::ir::ComparisonOperator::LessThanOrEqual, lhs, rhs)
        }
        ssf::ir::PrimitiveOperator::GreaterThan => {
            state.comparison_operation(fmm::ir::ComparisonOperator::GreaterThan, lhs, rhs)
        }
        ssf::ir::PrimitiveOperator::GreaterThanOrEqual => {
            state.comparison_operation(fmm::ir::ComparisonOperator::GreaterThanOrEqual, lhs, rhs)
        }
    }
}

fn compile_primitive(primitive: &ssf::ir::Primitive) -> fmm::ir::Primitive {
    match primitive {
        ssf::ir::Primitive::Float32(number) => fmm::ir::Primitive::Float32(*number),
        ssf::ir::Primitive::Float64(number) => fmm::ir::Primitive::Float64(*number),
        ssf::ir::Primitive::Integer8(number) => fmm::ir::Primitive::Integer8(*number),
        ssf::ir::Primitive::Integer32(number) => fmm::ir::Primitive::Integer32(*number),
        ssf::ir::Primitive::Integer64(number) => fmm::ir::Primitive::Integer64(*number),
    }
}
