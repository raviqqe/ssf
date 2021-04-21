use crate::closures;
use crate::entry_functions;
use crate::function_applications;
use crate::types;
use std::collections::HashMap;

pub fn compile_arity(arity: usize) -> fmm::ir::Primitive {
    fmm::ir::Primitive::PointerInteger(arity as i64)
}

pub fn compile(
    module_builder: &fmm::build::ModuleBuilder,
    instruction_builder: &fmm::build::InstructionBuilder,
    expression: &ssf::ir::Expression,
    variables: &HashMap<String, fmm::build::TypedExpression>,
) -> Result<fmm::build::TypedExpression, fmm::build::BuildError> {
    let compile =
        |expression, variables| compile(module_builder, instruction_builder, expression, variables);

    Ok(match expression {
        ssf::ir::Expression::ArithmeticOperation(operation) => {
            compile_arithmetic_operation(module_builder, instruction_builder, operation, variables)?
        }
        ssf::ir::Expression::Bitcast(bit_cast) => fmm::build::bit_cast(
            types::compile(bit_cast.type_()),
            compile(bit_cast.expression(), variables)?,
        )
        .into(),
        ssf::ir::Expression::Case(case) => {
            compile_case(module_builder, instruction_builder, case, variables)?
        }
        ssf::ir::Expression::ComparisonOperation(operation) => {
            compile_comparison_operation(module_builder, instruction_builder, operation, variables)?
        }
        ssf::ir::Expression::ConstructorApplication(constructor_application) => {
            let constructor = constructor_application.constructor();
            let algebraic_type = constructor.algebraic_type();
            let constructor_type =
                algebraic_type.unfold().constructors()[&constructor.tag()].clone();

            fmm::build::record(
                if algebraic_type.is_singleton() {
                    None
                } else {
                    Some(fmm::ir::Primitive::PointerInteger(constructor.tag() as i64).into())
                }
                .into_iter()
                .chain(if constructor_type.is_enum() {
                    None
                } else {
                    let payload = fmm::build::record(
                        constructor_application
                            .arguments()
                            .iter()
                            .map(|argument| compile(argument, variables))
                            .collect::<Result<_, _>>()?,
                    );
                    let union_type = types::compile_constructor_union(algebraic_type);
                    let member_index =
                        types::get_constructor_union_index(algebraic_type, constructor.tag());

                    Some(
                        fmm::ir::Union::new(
                            union_type.clone(),
                            member_index,
                            if constructor_type.is_boxed() {
                                let pointer =
                                    instruction_builder.allocate_heap(payload.type_().clone());
                                instruction_builder.store(payload, pointer.clone());

                                fmm::build::bit_cast(
                                    union_type.members()[member_index].clone(),
                                    pointer,
                                )
                                .expression()
                                .clone()
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
                module_builder,
                instruction_builder,
                compile(function_application.first_function(), variables)?,
                &function_application
                    .arguments()
                    .into_iter()
                    .map(|argument| compile(argument, variables))
                    .collect::<Result<Vec<_>, _>>()?,
            )?
        }
        ssf::ir::Expression::Let(let_) => {
            compile_let(module_builder, instruction_builder, let_, variables)?
        }
        ssf::ir::Expression::LetRecursive(let_recursive) => compile_let_recursive(
            module_builder,
            instruction_builder,
            let_recursive,
            variables,
        )?,
        ssf::ir::Expression::Primitive(primitive) => compile_primitive(primitive).into(),
        ssf::ir::Expression::Variable(variable) => variables[variable.name()].clone(),
    })
}

fn compile_case(
    module_builder: &fmm::build::ModuleBuilder,
    instruction_builder: &fmm::build::InstructionBuilder,
    case: &ssf::ir::Case,
    variables: &HashMap<String, fmm::build::TypedExpression>,
) -> Result<fmm::build::TypedExpression, fmm::build::BuildError> {
    let compile = |expression| compile(module_builder, instruction_builder, expression, variables);

    Ok(match case {
        ssf::ir::Case::Algebraic(case) => {
            let argument = compile(case.argument())?;

            compile_algebraic_alternatives(
                module_builder,
                instruction_builder,
                if case
                    .alternatives()
                    .get(0)
                    .map(|alternative| alternative.constructor().algebraic_type().is_singleton())
                    .unwrap_or(true)
                {
                    fmm::ir::Primitive::PointerInteger(0).into()
                } else {
                    instruction_builder.deconstruct_record(argument.clone(), 0)?
                },
                argument,
                case.alternatives(),
                case.default_alternative(),
                variables,
            )?
            .unwrap()
        }
        ssf::ir::Case::Primitive(case) => {
            compile_primitive_case(module_builder, instruction_builder, case, variables)?
        }
    })
}

fn compile_algebraic_alternatives(
    module_builder: &fmm::build::ModuleBuilder,
    instruction_builder: &fmm::build::InstructionBuilder,
    tag: fmm::build::TypedExpression,
    argument: fmm::build::TypedExpression,
    alternatives: &[ssf::ir::AlgebraicAlternative],
    default_alternative: Option<&ssf::ir::Expression>,
    variables: &HashMap<String, fmm::build::TypedExpression>,
) -> Result<Option<fmm::build::TypedExpression>, fmm::build::BuildError> {
    Ok(match alternatives {
        [] => default_alternative
            .map(|default_alternative| {
                compile(
                    module_builder,
                    instruction_builder,
                    default_alternative,
                    variables,
                )
            })
            .transpose()?,
        [alternative, ..] => {
            let constructor = alternative.constructor();

            Some(instruction_builder.if_(
                instruction_builder.comparison_operation(
                    fmm::ir::ComparisonOperator::Equal,
                    tag.clone(),
                    fmm::ir::Primitive::PointerInteger(constructor.tag() as i64),
                )?,
                |instruction_builder| {
                    Ok(instruction_builder.branch(compile(
                        module_builder,
                        &instruction_builder,
                        alternative.expression(),
                        &if constructor.constructor_type().is_enum() {
                            variables.clone()
                        } else {
                            let mut payload = instruction_builder.deconstruct_union(
                                instruction_builder.deconstruct_record(
                                    argument.clone(),
                                    if constructor.algebraic_type().is_singleton() {
                                        0
                                    } else {
                                        1
                                    },
                                )?,
                                types::get_constructor_union_index(
                                    constructor.algebraic_type(),
                                    constructor.tag(),
                                ),
                            )?;

                            if constructor.constructor_type().is_boxed() {
                                payload = instruction_builder.load(fmm::build::bit_cast(
                                    types::compile_boxed_constructor(
                                        constructor.constructor_type(),
                                    ),
                                    payload,
                                ))?;
                            }

                            variables
                                .clone()
                                .into_iter()
                                .chain(
                                    alternative
                                        .element_names()
                                        .iter()
                                        .enumerate()
                                        .map(|(index, name)| {
                                            Ok((
                                                name.into(),
                                                instruction_builder
                                                    .deconstruct_record(payload.clone(), index)?,
                                            ))
                                        })
                                        .collect::<Result<Vec<_>, _>>()?,
                                )
                                .collect()
                        },
                    )?))
                },
                |instruction_builder| {
                    Ok(
                        if let Some(expression) = compile_algebraic_alternatives(
                            module_builder,
                            &instruction_builder,
                            tag.clone(),
                            argument.clone(),
                            &alternatives[1..],
                            default_alternative,
                            variables,
                        )? {
                            instruction_builder.branch(expression)
                        } else {
                            instruction_builder.unreachable()
                        },
                    )
                },
            )?)
        }
    })
}

fn compile_primitive_case(
    module_builder: &fmm::build::ModuleBuilder,
    instruction_builder: &fmm::build::InstructionBuilder,
    case: &ssf::ir::PrimitiveCase,
    variables: &HashMap<String, fmm::build::TypedExpression>,
) -> Result<fmm::build::TypedExpression, fmm::build::BuildError> {
    let argument = compile(
        module_builder,
        instruction_builder,
        case.argument(),
        variables,
    )?;

    Ok(compile_primitive_alternatives(
        module_builder,
        instruction_builder,
        argument,
        case.alternatives(),
        case.default_alternative(),
        variables,
    )?
    .unwrap())
}

fn compile_primitive_alternatives(
    module_builder: &fmm::build::ModuleBuilder,
    instruction_builder: &fmm::build::InstructionBuilder,
    argument: fmm::build::TypedExpression,
    alternatives: &[ssf::ir::PrimitiveAlternative],
    default_alternative: Option<&ssf::ir::Expression>,
    variables: &HashMap<String, fmm::build::TypedExpression>,
) -> Result<Option<fmm::build::TypedExpression>, fmm::build::BuildError> {
    let compile = |expression| compile(module_builder, instruction_builder, expression, variables);

    Ok(match alternatives {
        [] => default_alternative.map(compile).transpose()?,
        [alternative, ..] => Some(instruction_builder.if_(
            instruction_builder.comparison_operation(
                fmm::ir::ComparisonOperator::Equal,
                argument.clone(),
                compile_primitive(alternative.primitive()),
            )?,
            |instruction_builder| {
                Ok(instruction_builder.branch(compile(alternative.expression())?))
            },
            |instruction_builder| {
                Ok(
                    if let Some(expression) = compile_primitive_alternatives(
                        module_builder,
                        &instruction_builder,
                        argument.clone(),
                        &alternatives[1..],
                        default_alternative,
                        variables,
                    )? {
                        instruction_builder.branch(expression)
                    } else {
                        instruction_builder.unreachable()
                    },
                )
            },
        )?),
    })
}

fn compile_let(
    module_builder: &fmm::build::ModuleBuilder,
    instruction_builder: &fmm::build::InstructionBuilder,
    let_: &ssf::ir::Let,
    variables: &HashMap<String, fmm::build::TypedExpression>,
) -> Result<fmm::build::TypedExpression, fmm::build::BuildError> {
    let compile =
        |expression, variables| compile(module_builder, instruction_builder, expression, variables);

    compile(
        let_.expression(),
        &variables
            .clone()
            .drain()
            .chain(vec![(
                let_.name().into(),
                compile(let_.bound_expression(), variables)?,
            )])
            .collect(),
    )
}

fn compile_let_recursive(
    module_builder: &fmm::build::ModuleBuilder,
    instruction_builder: &fmm::build::InstructionBuilder,
    let_: &ssf::ir::LetRecursive,
    variables: &HashMap<String, fmm::build::TypedExpression>,
) -> Result<fmm::build::TypedExpression, fmm::build::BuildError> {
    let mut variables = variables.clone();
    let mut closure_pointers = HashMap::new();

    for definition in let_.definitions() {
        let closure_pointer =
            instruction_builder.allocate_heap(types::compile_sized_closure(definition));

        variables.insert(
            definition.name().into(),
            fmm::build::bit_cast(
                fmm::types::Pointer::new(types::compile_unsized_closure(definition.type_())),
                closure_pointer.clone(),
            )
            .into(),
        );
        closure_pointers.insert(definition.name(), closure_pointer);
    }

    for definition in let_.definitions() {
        instruction_builder.store(
            closures::compile_closure_content(
                entry_functions::compile(module_builder, definition, &variables)?,
                definition
                    .environment()
                    .iter()
                    .map(|free_variable| variables[free_variable.name()].clone())
                    .collect(),
            ),
            closure_pointers[definition.name()].clone(),
        );
    }

    compile(
        module_builder,
        instruction_builder,
        let_.expression(),
        &variables,
    )
}

fn compile_arithmetic_operation(
    module_builder: &fmm::build::ModuleBuilder,
    instruction_builder: &fmm::build::InstructionBuilder,
    operation: &ssf::ir::ArithmeticOperation,
    variables: &HashMap<String, fmm::build::TypedExpression>,
) -> Result<fmm::build::TypedExpression, fmm::build::BuildError> {
    let compile = |expression| compile(module_builder, instruction_builder, expression, variables);

    let lhs = compile(operation.lhs())?;
    let rhs = compile(operation.rhs())?;

    Ok(match operation.operator() {
        ssf::ir::ArithmeticOperator::Add => {
            instruction_builder.arithmetic_operation(fmm::ir::ArithmeticOperator::Add, lhs, rhs)?
        }
        ssf::ir::ArithmeticOperator::Subtract => instruction_builder.arithmetic_operation(
            fmm::ir::ArithmeticOperator::Subtract,
            lhs,
            rhs,
        )?,
        ssf::ir::ArithmeticOperator::Multiply => instruction_builder.arithmetic_operation(
            fmm::ir::ArithmeticOperator::Multiply,
            lhs,
            rhs,
        )?,
        ssf::ir::ArithmeticOperator::Divide => instruction_builder.arithmetic_operation(
            fmm::ir::ArithmeticOperator::Divide,
            lhs,
            rhs,
        )?,
    })
}

fn compile_comparison_operation(
    module_builder: &fmm::build::ModuleBuilder,
    instruction_builder: &fmm::build::InstructionBuilder,
    operation: &ssf::ir::ComparisonOperation,
    variables: &HashMap<String, fmm::build::TypedExpression>,
) -> Result<fmm::build::TypedExpression, fmm::build::BuildError> {
    let compile = |expression| compile(module_builder, instruction_builder, expression, variables);

    let lhs = compile(operation.lhs())?;
    let rhs = compile(operation.rhs())?;

    instruction_builder.comparison_operation(
        match operation.operator() {
            ssf::ir::ComparisonOperator::Equal => fmm::ir::ComparisonOperator::Equal,
            ssf::ir::ComparisonOperator::NotEqual => fmm::ir::ComparisonOperator::NotEqual,
            ssf::ir::ComparisonOperator::GreaterThan => fmm::ir::ComparisonOperator::GreaterThan,
            ssf::ir::ComparisonOperator::GreaterThanOrEqual => {
                fmm::ir::ComparisonOperator::GreaterThanOrEqual
            }
            ssf::ir::ComparisonOperator::LessThan => fmm::ir::ComparisonOperator::LessThan,
            ssf::ir::ComparisonOperator::LessThanOrEqual => {
                fmm::ir::ComparisonOperator::LessThanOrEqual
            }
        },
        lhs,
        rhs,
    )
}

fn compile_primitive(primitive: &ssf::ir::Primitive) -> fmm::ir::Primitive {
    match primitive {
        ssf::ir::Primitive::Boolean(boolean) => fmm::ir::Primitive::Boolean(*boolean),
        ssf::ir::Primitive::Float32(number) => fmm::ir::Primitive::Float32(*number),
        ssf::ir::Primitive::Float64(number) => fmm::ir::Primitive::Float64(*number),
        ssf::ir::Primitive::Integer8(number) => fmm::ir::Primitive::Integer8(*number),
        ssf::ir::Primitive::Integer32(number) => fmm::ir::Primitive::Integer32(*number),
        ssf::ir::Primitive::Integer64(number) => fmm::ir::Primitive::Integer64(*number),
    }
}
