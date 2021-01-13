use super::utilities;
use crate::closures;
use crate::entry_functions;
use crate::function_applications;
use crate::names;
use crate::types;
use fmm::build::*;
use std::collections::HashMap;

pub fn compile_arity(arity: u64) -> fmm::ir::Primitive {
    fmm::ir::Primitive::PointerInteger(arity)
}

pub fn compile(
    expression: &ssf::ir::Expression,
    variables: &HashMap<String, BuildContext>,
) -> BuildContext {
    let compile = |expression| compile(expression, variables);

    match expression {
        ssf::ir::Expression::Bitcast(bitcast) => utilities::bitcast(
            compile(bitcast.expression()),
            types::compile(bitcast.type_()),
        ),
        ssf::ir::Expression::Case(case) => compile_case(case, variables),
        ssf::ir::Expression::ConstructorApplication(constructor_application) => {
            let constructor = constructor_application.constructor();
            let algebraic_type = constructor.algebraic_type();
            let constructor_type =
                algebraic_type.unfold().constructors()[&constructor.tag()].clone();

            let mut instructions = vec![];
            let mut payload = None;

            if !constructor_type.is_enum() {
                let arguments = vec![];

                for argument in constructor_application.arguments() {
                    let argument_context = compile(argument);

                    instructions.extend(argument_context.instructions().to_vec());
                    arguments.push(argument_context.expression().clone());
                }

                let record_type = types::compile_unboxed_constructor(&constructor_type);
                let unboxed_record = fmm::ir::Record::new(record_type, arguments);
                let record: fmm::ir::Expression = if constructor_type.is_boxed() {
                    let name = names::generate_name();

                    instructions.extend(vec![
                        fmm::ir::AllocateHeap::new(record_type, name).into(),
                        fmm::ir::Store::new(unboxed_record, fmm::ir::Variable::new(name)).into(),
                    ]);

                    fmm::ir::Variable::new(name).into()
                } else {
                    unboxed_record.into()
                };

                payload = Some(
                    fmm::ir::Union::new(
                        types::compile_untyped_constructor(algebraic_type),
                        types::get_constructor_union_index(algebraic_type, constructor.tag()),
                        record,
                    )
                    .into(),
                );
            }

            BuildContext::new(
                instructions,
                fmm::ir::Record::new(
                    types::compile_algebraic(algebraic_type, Some(constructor.tag())),
                    (if algebraic_type.is_singleton() {
                        None
                    } else {
                        Some(fmm::ir::Primitive::PointerInteger(constructor.tag()).into())
                    })
                    .into_iter()
                    .chain(payload)
                    .collect(),
                )
                .into(),
                types::compile_algebraic(algebraic_type, None),
            )
        }
        ssf::ir::Expression::FunctionApplication(function_application) => {
            let function_context = compile(function_application.first_function());

            let mut instructions = function_context.instructions().to_vec();
            let mut arguments = vec![];

            for argument in function_application.arguments() {
                let argument_context = compile(argument);

                instructions.extend(argument_context.instructions().iter().cloned());
                arguments.push(argument_context.clone_expression());
            }

            function_applications::compile(&function_context.clone_expression(), &arguments)
        }
        ssf::ir::Expression::Let(let_) => compile_let(let_, variables),
        ssf::ir::Expression::LetRecursive(let_recursive) => {
            compile_let_recursive(let_recursive, variables)
        }
        ssf::ir::Expression::Primitive(primitive) => compile_primitive(primitive).into(),
        ssf::ir::Expression::PrimitiveOperation(operation) => {
            compile_primitive_operation(operation, variables)
        }
        ssf::ir::Expression::Variable(variable) => {
            variables.get(variable.name()).unwrap().clone_expression()
        }
    }
}

fn compile_case(case: &ssf::ir::Case, variables: &HashMap<String, BuildContext>) -> BuildContext {
    let compile = |expression| compile(expression, variables);

    match case {
        ssf::ir::Case::Algebraic(algebraic_case) => {
            let context = compile(algebraic_case.argument());

            let tag_name = names::generate_name();
            let pointer_name = names::generate_name();
            let result_name = names::generate_name();

            (
                instructions
                    .into_iter()
                    .chain(vec![
                        (if algebraic_case
                            .alternatives()
                            .get(0)
                            .map(|alternative| {
                                alternative.constructor().algebraic_type().is_singleton()
                            })
                            .unwrap_or(true)
                        {
                            fmm::ir::Assignment::new(
                                fmm::ir::Primitive::PointerInteger(0),
                                &tag_name,
                            )
                            .into()
                        } else {
                            fmm::ir::DeconstructRecord::new(argument, 0, &tag_name).into()
                        }),
                        fmm::ir::AllocateStack::new(todo!(), &pointer_name).into(),
                        fmm::ir::Switch::new(
                            fmm::ir::Variable::new(tag_name),
                            algebraic_case
                                .alternatives()
                                .iter()
                                .map(|alternative| {
                                    let constructor = alternative.constructor();

                                    fmm::ir::Alternative::new(
                                        fmm::ir::Primitive::PointerInteger(constructor.tag()),
                                        {
                                            (if constructor.constructor_type().is_enum() {
                                                vec![]
                                            } else {
                                                let constructor_name = names::generate_name();

                                                vec![fmm::ir::DeconstructRecord::new(
                                                    argument,
                                                    if constructor.algebraic_type().is_singleton() {
                                                        0
                                                    } else {
                                                        1
                                                    },
                                                    &constructor_name,
                                                )
                                                .into()]
                                                .into_iter()
                                                .chain(
                                                    if constructor.constructor_type().is_boxed() {
                                                        Some(
                                                            fmm::ir::Load::new(
                                                                fmm::ir::Variable::new(
                                                                    constructor_name,
                                                                ),
                                                                constructor_name,
                                                            )
                                                            .into(),
                                                        )
                                                    } else {
                                                        None
                                                    },
                                                )
                                                .chain(
                                                    alternative
                                                        .element_names()
                                                        .iter()
                                                        .enumerate()
                                                        .map(|(index, name)| {
                                                            fmm::ir::DeconstructRecord::new(
                                                                fmm::ir::Variable::new(
                                                                    constructor_name,
                                                                ),
                                                                index,
                                                                name,
                                                            )
                                                            .into()
                                                        }),
                                                )
                                                .collect()
                                            })
                                            .into_iter()
                                            .chain({
                                                let (instructions, expression) =
                                                    compile(alternative.expression());

                                                instructions.into_iter().chain(vec![
                                                    fmm::ir::Store::new(
                                                        expression,
                                                        fmm::ir::Variable::new(pointer_name),
                                                    )
                                                    .into(),
                                                ])
                                            })
                                            .collect()
                                        },
                                    )
                                })
                                .collect(),
                            {
                                if let Some(expression) = algebraic_case.default_alternative() {
                                    let (instructions, expression) = compile(expression);

                                    instructions
                                        .into_iter()
                                        .chain(vec![fmm::ir::Store::new(
                                            expression,
                                            fmm::ir::Variable::new(pointer_name),
                                        )
                                        .into()])
                                        .collect()
                                } else {
                                    vec![fmm::ir::Instruction::Unreachable]
                                }
                            },
                        )
                        .into(),
                        fmm::ir::Load::new(fmm::ir::Variable::new(pointer_name), result_name)
                            .into(),
                    ])
                    .collect(),
                fmm::ir::Variable::new(result_name).into(),
            )
        }
        ssf::ir::Case::Primitive(case) => compile_primitive_case(case, variables),
    }
}

fn compile_primitive_case(
    case: &ssf::ir::PrimitiveCase,
    variables: &HashMap<String, BuildContext>,
) -> BuildContext {
    let argument_context = compile(case.argument(), variables);
    let alternatives_context = compile_primitive_alternatives(
        argument_context.clone_expression(),
        case.alternatives(),
        case.default_alternative(),
        variables,
    )
    .unwrap();

    BuildContext::new(
        argument_context
            .instructions()
            .iter()
            .cloned()
            .chain(alternatives_context.instructions().iter().cloned()),
        alternatives_context.expression().clone(),
        alternatives_context.type_().clone(),
    )
}

fn compile_primitive_alternatives(
    argument: BuildContext,
    alternatives: &[ssf::ir::PrimitiveAlternative],
    default_alternative: Option<&ssf::ir::Expression>,
    variables: &HashMap<String, BuildContext>,
) -> Option<BuildContext> {
    match alternatives {
        [] => default_alternative.map(|expression| compile(expression, variables)),
        [alternative, ..] => Some(utilities::if_(
            comparison_operation(
                fmm::ir::ComparisonOperator::Equal,
                argument.clone_expression(),
                compile_primitive(alternative.primitive()),
            ),
            branch(compile(alternative.expression(), variables)),
            compile_primitive_alternatives(
                argument,
                &alternatives[1..],
                default_alternative,
                variables,
            )
            .map(branch)
            .unwrap_or_else(unreachable),
        )),
    }
}

fn compile_let(let_: &ssf::ir::Let, variables: &HashMap<String, BuildContext>) -> BuildContext {
    let bound_expression_context = compile(let_.bound_expression(), variables);
    let expression_context = compile(
        let_.expression(),
        &variables
            .clone()
            .drain()
            .chain(vec![(
                let_.name().into(),
                bound_expression_context.clone_expression(),
            )])
            .collect(),
    );

    BuildContext::new(
        bound_expression_context
            .instructions()
            .iter()
            .cloned()
            .chain(expression_context.instructions().iter().cloned()),
        expression_context.expression().clone(),
        expression_context.type_().clone(),
    )
}

fn compile_let_recursive(
    let_: &ssf::ir::LetRecursive,
    variables: &HashMap<String, BuildContext>,
) -> BuildContext {
    let mut instructions = vec![];
    let mut variables = variables.clone();

    for definition in let_.definitions() {
        let context = allocate_heap(types::compile_sized_closure(definition));

        instructions.extend(context.instructions().iter().cloned());
        variables.insert(definition.name().into(), context.clone_expression());
    }

    for definition in let_.definitions() {
        instructions.extend(store(
            closures::compile_closure_content(
                &variable(
                    entry_functions::generate_closure_entry_name(definition.name()),
                    types::compile_entry_function_from_definition(definition),
                )
                .into(),
                &definition
                    .environment()
                    .iter()
                    .map(|free_variable| variables[free_variable.name()].clone_expression())
                    .collect::<Vec<_>>(),
            ),
            variables[definition.name()].clone_expression(),
        ));
    }

    let expression_context = compile(let_.expression(), &variables);

    BuildContext::new(
        instructions
            .iter()
            .cloned()
            .chain(expression_context.instructions().iter().cloned()),
        expression_context.expression().clone(),
        expression_context.type_().clone(),
    )
}

fn compile_primitive_operation(
    operation: &ssf::ir::PrimitiveOperation,
    variables: &HashMap<String, BuildContext>,
) -> BuildContext {
    let lhs = compile(operation.lhs(), variables);
    let rhs = compile(operation.rhs(), variables);

    match operation.operator() {
        ssf::ir::PrimitiveOperator::Add => {
            arithmetic_operation(fmm::ir::ArithmeticOperator::Add, lhs, rhs)
        }
        ssf::ir::PrimitiveOperator::Subtract => {
            arithmetic_operation(fmm::ir::ArithmeticOperator::Subtract, lhs, rhs)
        }
        ssf::ir::PrimitiveOperator::Multiply => {
            arithmetic_operation(fmm::ir::ArithmeticOperator::Multiply, lhs, rhs)
        }
        ssf::ir::PrimitiveOperator::Divide => {
            arithmetic_operation(fmm::ir::ArithmeticOperator::Divide, lhs, rhs)
        }
        ssf::ir::PrimitiveOperator::Equal => {
            comparison_operation(fmm::ir::ComparisonOperator::Equal, lhs, rhs)
        }
        ssf::ir::PrimitiveOperator::NotEqual => {
            comparison_operation(fmm::ir::ComparisonOperator::NotEqual, lhs, rhs)
        }
        ssf::ir::PrimitiveOperator::LessThan => {
            comparison_operation(fmm::ir::ComparisonOperator::LessThan, lhs, rhs)
        }
        ssf::ir::PrimitiveOperator::LessThanOrEqual => {
            comparison_operation(fmm::ir::ComparisonOperator::LessThanOrEqual, lhs, rhs)
        }
        ssf::ir::PrimitiveOperator::GreaterThan => {
            comparison_operation(fmm::ir::ComparisonOperator::GreaterThan, lhs, rhs)
        }
        ssf::ir::PrimitiveOperator::GreaterThanOrEqual => {
            comparison_operation(fmm::ir::ComparisonOperator::GreaterThanOrEqual, lhs, rhs)
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
