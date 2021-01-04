use crate::closures;
use crate::entry_functions;
use crate::function_applications;
use crate::types;

pub fn compile_arity(arity: u64) -> ssc::ir::Primitive {
    ssc::ir::Primitive::PointerInteger(arity)
}

pub fn compile(
    expression: &ssf::ir::Expression,
) -> (Vec<ssc::ir::Instruction>, ssc::ir::Expression) {
    match expression {
        ssf::ir::Expression::Bitcast(bitcast) => {
            let (instructions, expression) = compile(bitcast.expression());

            (
                instructions,
                ssc::ir::Bitcast::new(expression, types::compile(bitcast.type_())).into(),
            )
        }
        ssf::ir::Expression::Case(case) => compile_case(case),
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
                let record = ssc::ir::Record::new(record_type, arguments);

                const POINTER_NAME: &str = "_ptr";

                instructions.extend(vec![
                    ssc::ir::Assignment::new(POINTER_NAME, ssc::ir::Malloc::new(record_type))
                        .into(),
                    ssc::ir::Store::new(record, ssc::ir::Variable::new(POINTER_NAME)).into(),
                ]);

                payload = Some(
                    ssc::ir::Union::new(
                        types::compile_untyped_constructor(algebraic_type),
                        if constructor_type.is_boxed() {
                            ssc::ir::Variable::new(POINTER_NAME).into()
                        } else {
                            ssc::ir::Expression::from(record)
                        },
                    )
                    .into(),
                );
            }

            (
                instructions,
                ssc::ir::Record::new(
                    types::compile_algebraic(algebraic_type, Some(constructor.tag())),
                    (if algebraic_type.is_singleton() {
                        None
                    } else {
                        Some(ssc::ir::Primitive::PointerInteger(constructor.tag()).into())
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
        ssf::ir::Expression::Let(let_) => {
            let (bound_expression_instructions, bound_expression) =
                compile(let_.bound_expression());
            let (expression_instructions, expression) = compile(let_.expression());

            (
                bound_expression_instructions
                    .into_iter()
                    .chain(vec![ssc::ir::Assignment::new(
                        let_.name(),
                        bound_expression,
                    )
                    .into()])
                    .chain(expression_instructions)
                    .collect(),
                expression,
            )
        }
        ssf::ir::Expression::LetRecursive(let_recursive) => {
            let mut instructions = vec![];

            for definition in let_recursive.definitions() {
                instructions.push(
                    ssc::ir::Assignment::new(
                        definition.name(),
                        ssc::ir::Bitcast::new(
                            ssc::ir::Malloc::new(types::compile_sized_closure(definition)),
                            ssc::types::Pointer::new(types::compile_unsized_closure(
                                definition.type_(),
                            )),
                        ),
                    )
                    .into(),
                );
            }

            for definition in let_recursive.definitions() {
                instructions.push(
                    ssc::ir::Store::new(
                        closures::compile_closure_content(
                            &ssc::ir::Variable::new(entry_functions::generate_closure_entry_name(
                                definition.name(),
                            ))
                            .into(),
                            &definition
                                .environment()
                                .iter()
                                .map(|free_variable| {
                                    ssc::ir::Variable::new(free_variable.name()).into()
                                })
                                .collect::<Vec<_>>(),
                        ),
                        ssc::ir::Bitcast::new(
                            ssc::ir::Variable::new(definition.name()),
                            ssc::types::Pointer::new(types::compile_sized_closure(definition)),
                        ),
                    )
                    .into(),
                );
            }

            let (expression_instructions, expression) = compile(let_recursive.expression());

            (
                instructions
                    .into_iter()
                    .chain(expression_instructions)
                    .collect(),
                expression,
            )
        }
        ssf::ir::Expression::Primitive(primitive) => (vec![], compile_primitive(primitive).into()),
        ssf::ir::Expression::PrimitiveOperation(operation) => {
            compile_primitive_operation(operation)
        }
        ssf::ir::Expression::Variable(variable) => (vec![], compile_variable(variable).into()),
    }
}

fn compile_case(case: &ssf::ir::Case) -> (Vec<ssc::ir::Instruction>, ssc::ir::Expression) {
    match case {
        ssf::ir::Case::Algebraic(algebraic_case) => {
            let (mut instructions, argument) = compile(algebraic_case.argument());

            (
                instructions
                    .into_iter()
                    .chain(vec![ssc::ir::Switch::new(
                        if algebraic_case
                            .alternatives()
                            .get(0)
                            .map(|alternative| {
                                alternative.constructor().algebraic_type().is_singleton()
                            })
                            .unwrap_or(true)
                        {
                            ssc::ir::Primitive::PointerInteger(0).into()
                        } else {
                            ssc::ir::DeconstructRecord::new(argument, 0).into()
                        },
                        algebraic_case.alternatives().iter().map(|alternative| {
                            let block = append_basic_block("case");
                            position_at_end(block);

                            let mut variables = variables.clone();
                            let constructor = alternative.constructor();

                            if !constructor.constructor_type().is_enum() {
                                let argument = self
                                    .builder
                                    .build_load(
                                        builder
                                            .build_bitcast(
                                                argument_pointer,
                                                types::compile_algebraic(
                                                    constructor.algebraic_type(),
                                                    Some(constructor.tag()),
                                                )
                                                .ptr_type(ssc::AddressSpace::Generic),
                                                "",
                                            )
                                            .into_pointer_value(),
                                        "",
                                    )
                                    .into_struct_value();

                                let constructor_value = self
                                    .builder
                                    .build_extract_value(
                                        argument,
                                        if constructor.algebraic_type().is_singleton() {
                                            0
                                        } else {
                                            1
                                        },
                                        "",
                                    )
                                    .unwrap();

                                let constructor_value = if constructor.constructor_type().is_boxed()
                                {
                                    builder
                                        .build_load(constructor_value.into_pointer_value(), "")
                                        .into_struct_value()
                                } else {
                                    constructor_value.into_struct_value()
                                };

                                for (index, name) in alternative.element_names().iter().enumerate()
                                {
                                    variables.insert(
                                        name.into(),
                                        builder
                                            .build_extract_value(
                                                constructor_value,
                                                index as u32,
                                                "",
                                            )
                                            .unwrap(),
                                    );
                                }
                            }

                            let expression = compile(alternative.expression());

                            cases.push((
                                types::compile_tag().const_int(constructor.tag(), false),
                                block,
                                get_insert_block().unwrap(),
                                expression,
                            ));

                            build_unconditional_branch(phi_block);
                        }),
                        {
                            let mut default_case = None;
                            let default_block = append_basic_block("default");
                            position_at_end(default_block);

                            if let Some(expression) = algebraic_case.default_alternative() {
                                default_case =
                                    Some((compile(expression), get_insert_block().unwrap()));
                                build_unconditional_branch(phi_block);
                            } else {
                                build_unreachable();
                            }

                            position_at_end(switch_block);
                            build_switch(
                                tag,
                                default_block,
                                &cases
                                    .iter()
                                    .map(|(tag, start_block, _, _)| (*tag, *start_block))
                                    .collect::<Vec<_>>(),
                            );

                            let phi = build_phi(
                                cases
                                    .get(0)
                                    .map(|(_, _, _, value)| value.get_type())
                                    .unwrap_or_else(|| default_case.unwrap().0.get_type()),
                                "",
                            );
                            phi.add_incoming(
                                &cases
                                    .iter()
                                    .map(|(_, _, end_block, value)| {
                                        (value as &dyn ssc::ir::BasicValue, *end_block)
                                    })
                                    .chain(default_case.as_ref().map(|(value, block)| {
                                        (value as &dyn ssc::ir::BasicValue, *block)
                                    }))
                                    .collect::<Vec<_>>(),
                            );
                        },
                    )
                    .into()])
                    .collect(),
                expression,
            );
        }
        ssf::ir::Case::Primitive(primitive_case) => {
            let argument = compile(primitive_case.argument());

            let phi_block = append_basic_block("phi");
            let mut cases = vec![];

            for alternative in primitive_case.alternatives() {
                let then_block = append_basic_block("then");
                let else_block = append_basic_block("else");

                build_conditional_branch(
                    if argument.is_int_value() {
                        build_int_compare(
                            ssc::IntPredicate::EQ,
                            argument.into_int_value(),
                            compile_primitive(alternative.primitive()).into_int_value(),
                            "",
                        )
                    } else {
                        build_float_compare(
                            ssc::FloatPredicate::OEQ,
                            argument.into_float_value(),
                            compile_primitive(alternative.primitive()).into_float_value(),
                            "",
                        )
                    },
                    then_block,
                    else_block,
                );
                position_at_end(then_block);

                cases.push((
                    compile(alternative.expression(), &variables),
                    get_insert_block().unwrap(),
                ));

                build_unconditional_branch(phi_block);
                position_at_end(else_block);
            }

            if let Some(expression) = primitive_case.default_alternative() {
                cases.push((compile(expression, &variables), get_insert_block().unwrap()));
                build_unconditional_branch(phi_block);
            } else {
                build_unreachable();
            }

            position_at_end(phi_block);
            let phi = build_phi(cases[0].0.get_type(), "");
            phi.add_incoming(
                &cases
                    .iter()
                    .map(|(value, block)| (value as &dyn ssc::ir::BasicValue, *block))
                    .collect::<Vec<_>>(),
            );

            phi.as_basic_value()
        }
    }
}

fn compile_primitive_operation(
    operation: &ssf::ir::PrimitiveOperation,
) -> (Vec<ssc::ir::Instruction>, ssc::ir::Expression) {
    let (lhs_instructions, lhs) = compile(operation.lhs());
    let (rhs_instructions, rhs) = compile(operation.rhs());

    (
        lhs_instructions
            .into_iter()
            .chain(rhs_instructions)
            .collect(),
        ssc::ir::PrimitiveOperation::new(
            compile_primitive_operator(operation.operator()),
            lhs,
            rhs,
        )
        .into(),
    )
}

fn compile_primitive_operator(operator: ssf::ir::PrimitiveOperator) -> ssc::ir::PrimitiveOperator {
    match operator {
        ssf::ir::PrimitiveOperator::Add => ssc::ir::PrimitiveOperator::Add,
        ssf::ir::PrimitiveOperator::Subtract => ssc::ir::PrimitiveOperator::Subtract,
        ssf::ir::PrimitiveOperator::Multiply => ssc::ir::PrimitiveOperator::Multiply,
        ssf::ir::PrimitiveOperator::Divide => ssc::ir::PrimitiveOperator::Divide,
        ssf::ir::PrimitiveOperator::Equal => ssc::ir::PrimitiveOperator::Equal,
        ssf::ir::PrimitiveOperator::NotEqual => ssc::ir::PrimitiveOperator::NotEqual,
        ssf::ir::PrimitiveOperator::LessThan => ssc::ir::PrimitiveOperator::LessThan,
        ssf::ir::PrimitiveOperator::LessThanOrEqual => ssc::ir::PrimitiveOperator::LessThanOrEqual,
        ssf::ir::PrimitiveOperator::GreaterThan => ssc::ir::PrimitiveOperator::GreaterThan,
        ssf::ir::PrimitiveOperator::GreaterThanOrEqual => {
            ssc::ir::PrimitiveOperator::GreaterThanOrEqual
        }
    }
}

fn compile_primitive(primitive: &ssf::ir::Primitive) -> ssc::ir::Primitive {
    match primitive {
        ssf::ir::Primitive::Float32(number) => ssc::ir::Primitive::Float32(*number),
        ssf::ir::Primitive::Float64(number) => ssc::ir::Primitive::Float64(*number),
        ssf::ir::Primitive::Integer8(number) => ssc::ir::Primitive::Integer8(*number),
        ssf::ir::Primitive::Integer32(number) => ssc::ir::Primitive::Integer32(*number),
        ssf::ir::Primitive::Integer64(number) => ssc::ir::Primitive::Integer64(*number),
    }
}

fn compile_variable(variable: &ssf::ir::Variable) -> ssc::ir::Variable {
    ssc::ir::Variable::new(variable.name())
}
