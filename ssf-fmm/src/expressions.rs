use crate::closures;
use crate::entry_functions;
use crate::function_applications;
use crate::names;
use crate::types;
use std::collections::HashMap;

pub fn compile_arity(arity: u64) -> fmm::ir::Primitive {
    fmm::ir::Primitive::PointerInteger(arity)
}

pub fn compile(
    expression: &ssf::ir::Expression,
    variables: &HashMap<String, ssf::types::Type>,
) -> fmm::build::InstructionContext {
    let compile = |expression| compile(expression, variables);

    match expression {
        ssf::ir::Expression::Bitcast(bitcast) => {
            let (instructions, expression) = compile(bitcast.expression());
            let name = names::generate_name();

            (
                instructions
                    .into_iter()
                    .chain(vec![fmm::ir::Bitcast::new(
                        expression,
                        types::compile(bitcast.from()),
                        types::compile(bitcast.type_()),
                        name,
                    )
                    .into()])
                    .collect(),
                fmm::ir::Variable::new(name).into(),
            )
        }
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
                    let (argument_instructions, argument) = compile(argument);

                    instructions.extend(argument_instructions);
                    arguments.push(argument);
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

            (
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
            )
        }
        ssf::ir::Expression::FunctionApplication(function_application) => {
            let (mut instructions, function) = compile(function_application.first_function());
            let mut arguments = vec![];

            for argument in function_application.arguments() {
                let (argument_instructions, argument) = compile(argument);

                instructions.extend(argument_instructions);
                arguments.push(argument);
            }

            let (application_instructions, expression) =
                function_applications::compile(&function, &arguments);

            (
                instructions
                    .into_iter()
                    .chain(application_instructions)
                    .collect(),
                expression,
            )
        }
        ssf::ir::Expression::Let(let_) => compile_let(let_, variables),
        ssf::ir::Expression::LetRecursive(let_recursive) => {
            compile_let_recursive(let_recursive, variables)
        }
        ssf::ir::Expression::Primitive(primitive) => (vec![], compile_primitive(primitive).into()),
        ssf::ir::Expression::PrimitiveOperation(operation) => {
            compile_primitive_operation(operation, variables)
        }
        ssf::ir::Expression::Variable(variable) => (vec![], compile_variable(variable).into()),
    }
}

fn compile_case(
    case: &ssf::ir::Case,
    variables: &HashMap<String, ssf::types::Type>,
) -> (Vec<fmm::ir::Instruction>, fmm::ir::Expression) {
    let compile = |expression| compile(expression, variables);

    match case {
        ssf::ir::Case::Algebraic(algebraic_case) => {
            let (instructions, argument) = compile(algebraic_case.argument());

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
        ssf::ir::Case::Primitive(case) => {
            let pointer_name = names::generate_name();
            let result_name = names::generate_name();
            let (instructions, argument) = compile(case.argument());

            (
                instructions
                    .into_iter()
                    .chain(vec![
                        fmm::ir::AllocateStack::new(todo!(), pointer_name).into()
                    ])
                    .chain(case.alternatives().iter().rev().fold(
                        if let Some(expression) = case.default_alternative() {
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
                        },
                        |instructions, alternative| {
                            let condition_name = names::generate_name();

                            vec![
                                fmm::ir::PrimitiveOperation::new(
                                    fmm::ir::PrimitiveOperator::Equal,
                                    argument,
                                    compile_primitive(alternative.primitive()),
                                    &condition_name,
                                )
                                .into(),
                                fmm::ir::If::new(
                                    fmm::ir::Variable::new(condition_name),
                                    {
                                        let (instructions, expression) =
                                            compile(alternative.expression());

                                        instructions
                                            .into_iter()
                                            .chain(vec![fmm::ir::Store::new(
                                                expression,
                                                fmm::ir::Variable::new(pointer_name),
                                            )
                                            .into()])
                                            .collect()
                                    },
                                    instructions,
                                )
                                .into(),
                            ]
                        },
                    ))
                    .collect(),
                fmm::ir::Variable::new(result_name).into(),
            )
        }
    }
}

fn compile_let(
    let_: &ssf::ir::Let,
    variables: &HashMap<String, ssf::types::Type>,
) -> (Vec<fmm::ir::Instruction>, fmm::ir::Expression) {
    let (bound_expression_instructions, bound_expression) =
        compile(let_.bound_expression(), variables);

    let (expression_instructions, expression) = compile(
        let_.expression(),
        &variables
            .clone()
            .drain()
            .chain(vec![(let_.name().into(), let_.type_().clone())])
            .collect(),
    );

    (
        bound_expression_instructions
            .into_iter()
            .chain(vec![fmm::ir::Assignment::new(
                bound_expression,
                let_.name(),
            )
            .into()])
            .chain(expression_instructions)
            .collect(),
        expression,
    )
}

fn compile_let_recursive(
    let_: &ssf::ir::LetRecursive,
    variables: &HashMap<String, ssf::types::Type>,
) -> (Vec<fmm::ir::Instruction>, fmm::ir::Expression) {
    let variables = variables
        .clone()
        .drain()
        .chain(
            let_.definitions()
                .iter()
                .map(|definition| (definition.name().into(), definition.type_().clone().into())),
        )
        .collect();

    let (expression_instructions, expression) = compile(let_.expression(), &variables);

    (
        let_.definitions()
            .iter()
            .flat_map(|definition| {
                let name = names::generate_name();

                vec![
                    fmm::ir::AllocateHeap::new(types::compile_sized_closure(definition), name)
                        .into(),
                    fmm::ir::Bitcast::new(
                        fmm::ir::Variable::new(name),
                        fmm::types::Pointer::new(types::compile_unsized_closure(
                            definition.type_(),
                        )),
                        definition.name(),
                    )
                    .into(),
                ]
            })
            .chain(let_.definitions().iter().flat_map(|definition| {
                let name = names::generate_name();

                vec![
                    fmm::ir::Bitcast::new(
                        fmm::ir::Variable::new(definition.name()),
                        fmm::types::Pointer::new(types::compile_sized_closure(definition)),
                        name,
                    )
                    .into(),
                    fmm::ir::Store::new(
                        closures::compile_closure_content(
                            &fmm::ir::Variable::new(entry_functions::generate_closure_entry_name(
                                definition.name(),
                            ))
                            .into(),
                            &definition
                                .environment()
                                .iter()
                                .map(|free_variable| {
                                    fmm::ir::Variable::new(free_variable.name()).into()
                                })
                                .collect::<Vec<_>>(),
                        ),
                        fmm::ir::Variable::new(name),
                    )
                    .into(),
                ]
            }))
            .chain(expression_instructions)
            .collect(),
        expression,
    )
}

fn compile_primitive_operation(
    operation: &ssf::ir::PrimitiveOperation,
    variables: &HashMap<String, ssf::types::Type>,
) -> (Vec<fmm::ir::Instruction>, fmm::ir::Expression) {
    let (lhs_instructions, lhs) = compile(operation.lhs(), variables);
    let (rhs_instructions, rhs) = compile(operation.rhs(), variables);
    let name = names::generate_name();

    (
        lhs_instructions
            .into_iter()
            .chain(rhs_instructions)
            .chain(vec![fmm::ir::PrimitiveOperation::new(
                compile_primitive_operator(operation.operator()),
                lhs,
                rhs,
                name,
            )
            .into()])
            .collect(),
        fmm::ir::Variable::new(name).into(),
    )
}

fn compile_primitive_operator(operator: ssf::ir::PrimitiveOperator) -> fmm::ir::PrimitiveOperator {
    match operator {
        ssf::ir::PrimitiveOperator::Add => fmm::ir::PrimitiveOperator::Add,
        ssf::ir::PrimitiveOperator::Subtract => fmm::ir::PrimitiveOperator::Subtract,
        ssf::ir::PrimitiveOperator::Multiply => fmm::ir::PrimitiveOperator::Multiply,
        ssf::ir::PrimitiveOperator::Divide => fmm::ir::PrimitiveOperator::Divide,
        ssf::ir::PrimitiveOperator::Equal => fmm::ir::PrimitiveOperator::Equal,
        ssf::ir::PrimitiveOperator::NotEqual => fmm::ir::PrimitiveOperator::NotEqual,
        ssf::ir::PrimitiveOperator::LessThan => fmm::ir::PrimitiveOperator::LessThan,
        ssf::ir::PrimitiveOperator::LessThanOrEqual => fmm::ir::PrimitiveOperator::LessThanOrEqual,
        ssf::ir::PrimitiveOperator::GreaterThan => fmm::ir::PrimitiveOperator::GreaterThan,
        ssf::ir::PrimitiveOperator::GreaterThanOrEqual => {
            fmm::ir::PrimitiveOperator::GreaterThanOrEqual
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

fn compile_variable(variable: &ssf::ir::Variable) -> fmm::ir::Variable {
    fmm::ir::Variable::new(variable.name())
}
