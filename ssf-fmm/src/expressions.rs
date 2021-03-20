use crate::closures;
use crate::entry_functions;
use crate::function_applications;
use crate::types;
use crate::utilities;
use crate::variable_builder::VariableBuilder;
use std::collections::HashMap;

pub fn compile_arity(arity: usize) -> fmm::ir::Primitive {
    fmm::ir::Primitive::PointerInteger(arity as i64)
}

pub fn compile(
    module_builder: &fmm::build::ModuleBuilder,
    instruction_builder: &fmm::build::InstructionBuilder,
    expression: &ssf::ir::Expression,
    variables: &HashMap<String, VariableBuilder>,
) -> fmm::build::TypedExpression {
    let compile =
        |expression, variables| compile(module_builder, instruction_builder, expression, variables);

    match expression {
        ssf::ir::Expression::ArithmeticOperation(operation) => {
            compile_arithmetic_operation(module_builder, instruction_builder, operation, variables)
        }
        ssf::ir::Expression::Bitcast(bitcast) => utilities::bitcast(
            instruction_builder,
            compile(bitcast.expression(), variables),
            types::compile(bitcast.type_()),
        ),
        ssf::ir::Expression::Case(case) => {
            compile_case(module_builder, instruction_builder, case, variables)
        }
        ssf::ir::Expression::ComparisonOperation(operation) => {
            compile_comparison_operation(module_builder, instruction_builder, operation, variables)
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
                            .collect(),
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

                                utilities::bitcast(
                                    &instruction_builder,
                                    pointer,
                                    union_type.members()[member_index].clone(),
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
                compile(function_application.first_function(), variables),
                &function_application
                    .arguments()
                    .into_iter()
                    .map(|argument| compile(argument, variables))
                    .collect::<Vec<_>>(),
            )
        }
        ssf::ir::Expression::Let(let_) => {
            compile_let(module_builder, instruction_builder, let_, variables)
        }
        ssf::ir::Expression::LetRecursive(let_recursive) => compile_let_recursive(
            module_builder,
            instruction_builder,
            let_recursive,
            variables,
        ),
        ssf::ir::Expression::Primitive(primitive) => compile_primitive(primitive).into(),
        ssf::ir::Expression::Variable(variable) => variables
            .get(variable.name())
            .unwrap()
            .build(instruction_builder),
    }
}

fn compile_case(
    module_builder: &fmm::build::ModuleBuilder,
    instruction_builder: &fmm::build::InstructionBuilder,
    case: &ssf::ir::Case,
    variables: &HashMap<String, VariableBuilder>,
) -> fmm::build::TypedExpression {
    let compile = |expression| compile(module_builder, instruction_builder, expression, variables);

    match case {
        ssf::ir::Case::Algebraic(case) => {
            let argument = compile(case.argument());

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
                    instruction_builder.deconstruct_record(argument.clone(), 0)
                },
                argument,
                case.alternatives(),
                case.default_alternative(),
                variables,
            )
            .unwrap()
        }
        ssf::ir::Case::Primitive(case) => {
            compile_primitive_case(module_builder, instruction_builder, case, variables)
        }
    }
}

fn compile_algebraic_alternatives(
    module_builder: &fmm::build::ModuleBuilder,
    instruction_builder: &fmm::build::InstructionBuilder,
    tag: fmm::build::TypedExpression,
    argument: fmm::build::TypedExpression,
    alternatives: &[ssf::ir::AlgebraicAlternative],
    default_alternative: Option<&ssf::ir::Expression>,
    variables: &HashMap<String, VariableBuilder>,
) -> Option<fmm::build::TypedExpression> {
    Some(match alternatives {
        [] => compile(
            module_builder,
            instruction_builder,
            default_alternative?,
            variables,
        ),
        [alternative, ..] => {
            let constructor = alternative.constructor();

            instruction_builder.if_(
                instruction_builder.comparison_operation(
                    fmm::ir::ComparisonOperator::Equal,
                    tag.clone(),
                    fmm::ir::Primitive::PointerInteger(constructor.tag() as i64),
                ),
                |instruction_builder| {
                    instruction_builder.branch(compile(
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
                                ),
                                types::get_constructor_union_index(
                                    constructor.algebraic_type(),
                                    constructor.tag(),
                                ),
                            );

                            if constructor.constructor_type().is_boxed() {
                                payload = instruction_builder.load(utilities::bitcast(
                                    &instruction_builder,
                                    payload,
                                    types::compile_boxed_constructor(
                                        constructor.constructor_type(),
                                    ),
                                ));
                            }

                            variables
                                .clone()
                                .into_iter()
                                .chain(alternative.element_names().iter().enumerate().map(
                                    |(index, name)| {
                                        (
                                            name.into(),
                                            instruction_builder
                                                .deconstruct_record(payload.clone(), index)
                                                .into(),
                                        )
                                    },
                                ))
                                .collect()
                        },
                    ))
                },
                |instruction_builder| {
                    if let Some(expression) = compile_algebraic_alternatives(
                        module_builder,
                        &instruction_builder,
                        tag.clone(),
                        argument.clone(),
                        &alternatives[1..],
                        default_alternative,
                        variables,
                    ) {
                        instruction_builder.branch(expression)
                    } else {
                        instruction_builder.unreachable()
                    }
                },
            )
        }
    })
}

fn compile_primitive_case(
    module_builder: &fmm::build::ModuleBuilder,
    instruction_builder: &fmm::build::InstructionBuilder,
    case: &ssf::ir::PrimitiveCase,
    variables: &HashMap<String, VariableBuilder>,
) -> fmm::build::TypedExpression {
    let argument = compile(
        module_builder,
        instruction_builder,
        case.argument(),
        variables,
    );

    compile_primitive_alternatives(
        module_builder,
        instruction_builder,
        argument,
        case.alternatives(),
        case.default_alternative(),
        variables,
    )
    .unwrap()
}

fn compile_primitive_alternatives(
    module_builder: &fmm::build::ModuleBuilder,
    instruction_builder: &fmm::build::InstructionBuilder,
    argument: fmm::build::TypedExpression,
    alternatives: &[ssf::ir::PrimitiveAlternative],
    default_alternative: Option<&ssf::ir::Expression>,
    variables: &HashMap<String, VariableBuilder>,
) -> Option<fmm::build::TypedExpression> {
    let compile = |expression| compile(module_builder, instruction_builder, expression, variables);

    match alternatives {
        [] => default_alternative.map(compile),
        [alternative, ..] => Some(instruction_builder.if_(
            instruction_builder.comparison_operation(
                fmm::ir::ComparisonOperator::Equal,
                argument.clone(),
                compile_primitive(alternative.primitive()),
            ),
            |instruction_builder| instruction_builder.branch(compile(alternative.expression())),
            |instruction_builder| {
                if let Some(expression) = compile_primitive_alternatives(
                    module_builder,
                    &instruction_builder,
                    argument.clone(),
                    &alternatives[1..],
                    default_alternative,
                    variables,
                ) {
                    instruction_builder.branch(expression)
                } else {
                    instruction_builder.unreachable()
                }
            },
        )),
    }
}

fn compile_let(
    module_builder: &fmm::build::ModuleBuilder,
    instruction_builder: &fmm::build::InstructionBuilder,
    let_: &ssf::ir::Let,
    variables: &HashMap<String, VariableBuilder>,
) -> fmm::build::TypedExpression {
    let compile =
        |expression, variables| compile(module_builder, instruction_builder, expression, variables);

    compile(
        let_.expression(),
        &variables
            .clone()
            .drain()
            .chain(vec![(
                let_.name().into(),
                compile(let_.bound_expression(), variables).into(),
            )])
            .collect(),
    )
}

fn compile_let_recursive(
    module_builder: &fmm::build::ModuleBuilder,
    instruction_builder: &fmm::build::InstructionBuilder,
    let_: &ssf::ir::LetRecursive,
    variables: &HashMap<String, VariableBuilder>,
) -> fmm::build::TypedExpression {
    let mut variables = variables.clone();
    let mut closure_pointers = HashMap::new();

    for definition in let_.definitions() {
        let closure_pointer =
            instruction_builder.allocate_heap(types::compile_sized_closure(definition));

        variables.insert(
            definition.name().into(),
            utilities::bitcast(
                instruction_builder,
                closure_pointer.clone(),
                fmm::types::Pointer::new(types::compile_unsized_closure(definition.type_())),
            )
            .into(),
        );
        closure_pointers.insert(definition.name(), closure_pointer);
    }

    for definition in let_.definitions() {
        instruction_builder.store(
            closures::compile_closure_content(
                entry_functions::compile(module_builder, definition, &variables),
                definition
                    .environment()
                    .iter()
                    .map(|free_variable| variables[free_variable.name()].build(instruction_builder))
                    .collect::<Vec<_>>(),
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
    variables: &HashMap<String, VariableBuilder>,
) -> fmm::build::TypedExpression {
    let compile = |expression| compile(module_builder, instruction_builder, expression, variables);

    let lhs = compile(operation.lhs());
    let rhs = compile(operation.rhs());

    match operation.operator() {
        ssf::ir::ArithmeticOperator::Add => {
            instruction_builder.arithmetic_operation(fmm::ir::ArithmeticOperator::Add, lhs, rhs)
        }
        ssf::ir::ArithmeticOperator::Subtract => instruction_builder.arithmetic_operation(
            fmm::ir::ArithmeticOperator::Subtract,
            lhs,
            rhs,
        ),
        ssf::ir::ArithmeticOperator::Multiply => instruction_builder.arithmetic_operation(
            fmm::ir::ArithmeticOperator::Multiply,
            lhs,
            rhs,
        ),
        ssf::ir::ArithmeticOperator::Divide => {
            instruction_builder.arithmetic_operation(fmm::ir::ArithmeticOperator::Divide, lhs, rhs)
        }
    }
}

fn compile_comparison_operation(
    module_builder: &fmm::build::ModuleBuilder,
    instruction_builder: &fmm::build::InstructionBuilder,
    operation: &ssf::ir::ComparisonOperation,
    variables: &HashMap<String, VariableBuilder>,
) -> fmm::build::TypedExpression {
    let compile = |expression| compile(module_builder, instruction_builder, expression, variables);

    let lhs = compile(operation.lhs());
    let rhs = compile(operation.rhs());

    match operation.operator() {
        ssf::ir::ComparisonOperator::Equal => {
            instruction_builder.comparison_operation(fmm::ir::ComparisonOperator::Equal, lhs, rhs)
        }
        ssf::ir::ComparisonOperator::NotEqual => instruction_builder.comparison_operation(
            fmm::ir::ComparisonOperator::NotEqual,
            lhs,
            rhs,
        ),
        ssf::ir::ComparisonOperator::LessThan => instruction_builder.comparison_operation(
            fmm::ir::ComparisonOperator::LessThan,
            lhs,
            rhs,
        ),
        ssf::ir::ComparisonOperator::LessThanOrEqual => instruction_builder.comparison_operation(
            fmm::ir::ComparisonOperator::LessThanOrEqual,
            lhs,
            rhs,
        ),
        ssf::ir::ComparisonOperator::GreaterThan => instruction_builder.comparison_operation(
            fmm::ir::ComparisonOperator::GreaterThan,
            lhs,
            rhs,
        ),
        ssf::ir::ComparisonOperator::GreaterThanOrEqual => instruction_builder
            .comparison_operation(fmm::ir::ComparisonOperator::GreaterThanOrEqual, lhs, rhs),
    }
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
