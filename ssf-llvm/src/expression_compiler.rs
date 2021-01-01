use super::closure_operation_compiler::ClosureOperationCompiler;
use super::error::CompileError;
use super::function_application_compiler::FunctionApplicationCompiler;
use super::function_compiler::FunctionCompiler;
use super::malloc_compiler::MallocCompiler;
use super::type_compiler::TypeCompiler;
use inkwell::types::AnyType;
use std::collections::HashMap;
use std::sync::Arc;

pub struct ExpressionCompiler<'c> {
    context: &'c inkwell::context::Context,
    builder: Arc<inkwell::builder::Builder<'c>>,
    function_compiler: Arc<FunctionCompiler<'c>>,
    function_application_compiler: Arc<FunctionApplicationCompiler<'c>>,
    type_compiler: Arc<TypeCompiler<'c>>,
    closure_operation_compiler: Arc<ClosureOperationCompiler<'c>>,
    malloc_compiler: Arc<MallocCompiler<'c>>,
}

impl<'c> ExpressionCompiler<'c> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        context: &'c inkwell::context::Context,
        builder: Arc<inkwell::builder::Builder<'c>>,
        function_compiler: Arc<FunctionCompiler<'c>>,
        function_application_compiler: Arc<FunctionApplicationCompiler<'c>>,
        type_compiler: Arc<TypeCompiler<'c>>,
        closure_operation_compiler: Arc<ClosureOperationCompiler<'c>>,
        malloc_compiler: Arc<MallocCompiler<'c>>,
    ) -> Arc<Self> {
        Self {
            context,
            builder,
            function_compiler,
            function_application_compiler,
            type_compiler,
            closure_operation_compiler,
            malloc_compiler,
        }
        .into()
    }

    pub fn compile(
        &self,
        expression: &ssf::ir::Expression,
        variables: &HashMap<String, inkwell::values::BasicValueEnum<'c>>,
    ) -> Result<inkwell::values::BasicValueEnum<'c>, CompileError> {
        Ok(match expression {
            ssf::ir::Expression::Bitcast(bitcast) => {
                let argument = self.compile(bitcast.expression(), variables)?;
                let to_type = self.type_compiler.compile(bitcast.type_());

                if self.is_bitcast_supported(argument.get_type())
                    && self.is_bitcast_supported(to_type)
                {
                    self.builder.build_bitcast(
                        argument,
                        self.type_compiler.compile(bitcast.type_()),
                        "",
                    )
                } else if self
                    .type_compiler
                    .equal_bit_sizes(argument.get_type(), to_type)
                {
                    let pointer = self.builder.build_alloca(argument.get_type(), "");
                    self.builder.build_store(pointer, argument);

                    // TODO Use BasicTypeEnum::ptr_type() when it's implemented.
                    self.builder.build_load(
                        self.builder
                            .build_bitcast(
                                pointer,
                                self.builder.build_alloca(to_type, "").get_type(),
                                "",
                            )
                            .into_pointer_value(),
                        "",
                    )
                } else {
                    return Err(CompileError::InvalidBitcast(
                        argument.get_type().print_to_string().to_string(),
                        to_type.print_to_string().to_string(),
                    ));
                }
            }
            ssf::ir::Expression::Case(case) => self.compile_case(case, variables)?,
            ssf::ir::Expression::ConstructorApplication(constructor_application) => {
                let constructor = constructor_application.constructor();
                let algebraic_type = constructor.algebraic_type();
                let constructor_type =
                    algebraic_type.unfold().constructors()[&constructor.tag()].clone();

                let mut algebraic_value = self
                    .type_compiler
                    .compile_algebraic(algebraic_type, Some(constructor.tag()))
                    .const_zero()
                    .into();

                if !algebraic_type.is_singleton() {
                    algebraic_value = self
                        .builder
                        .build_insert_value(
                            algebraic_value,
                            self.type_compiler
                                .compile_tag()
                                .const_int(constructor.tag(), false),
                            0,
                            "",
                        )
                        .unwrap();
                }

                if !constructor_type.is_enum() {
                    let constructor_type = self
                        .type_compiler
                        .compile_unboxed_constructor(&constructor_type);

                    let mut constructor_value = constructor_type.const_zero().into();

                    for (index, argument) in constructor_application.arguments().iter().enumerate()
                    {
                        constructor_value = self
                            .builder
                            .build_insert_value(
                                constructor_value,
                                self.compile(argument, variables)?,
                                index as u32,
                                "",
                            )
                            .unwrap();
                    }

                    let constructor_value: inkwell::values::BasicValueEnum<'c> =
                        if constructor.constructor_type().is_boxed() {
                            let constructor_pointer = self.compile_malloc(constructor_type);

                            self.builder
                                .build_store(constructor_pointer, constructor_value);

                            constructor_pointer.into()
                        } else {
                            constructor_value.into_struct_value().into()
                        };

                    algebraic_value = self
                        .builder
                        .build_insert_value(
                            algebraic_value,
                            constructor_value,
                            if algebraic_type.is_singleton() { 0 } else { 1 },
                            "",
                        )
                        .unwrap();
                }

                let algebraic_pointer = self.builder.build_alloca(
                    self.type_compiler.compile_algebraic(algebraic_type, None),
                    "",
                );

                self.builder.build_store(
                    self.builder
                        .build_bitcast(
                            algebraic_pointer,
                            algebraic_value
                                .into_struct_value()
                                .get_type()
                                .ptr_type(inkwell::AddressSpace::Generic),
                            "",
                        )
                        .into_pointer_value(),
                    algebraic_value,
                );

                self.builder.build_load(algebraic_pointer, "")
            }
            ssf::ir::Expression::FunctionApplication(function_application) => {
                self.function_application_compiler.compile(
                    &self.builder,
                    self.compile(function_application.first_function(), variables)?
                        .into_pointer_value(),
                    &function_application
                        .arguments()
                        .into_iter()
                        .map(|argument| self.compile(argument, variables))
                        .collect::<Result<Vec<_>, _>>()?,
                )?
            }
            ssf::ir::Expression::LetRecursive(let_recursive) => {
                let mut variables = variables.clone();
                let mut closures = HashMap::<&str, inkwell::values::PointerValue>::new();

                for definition in let_recursive.definitions() {
                    let pointer =
                        self.compile_malloc(self.type_compiler.compile_sized_closure(definition));

                    variables.insert(
                        definition.name().into(),
                        self.builder.build_bitcast(
                            pointer,
                            self.type_compiler
                                .compile_unsized_closure(definition.type_())
                                .ptr_type(inkwell::AddressSpace::Generic),
                            "",
                        ),
                    );
                    closures.insert(definition.name(), pointer);
                }

                for definition in let_recursive.definitions() {
                    let closure = closures[definition.name()];

                    self.closure_operation_compiler
                        .compile_store_closure_content(
                            &self.builder,
                            closure,
                            self.function_compiler.compile(definition)?,
                            &definition
                                .environment()
                                .iter()
                                .map(|argument| {
                                    variables.get(argument.name()).copied().ok_or_else(|| {
                                        CompileError::VariableNotFound(argument.name().into())
                                    })
                                })
                                .collect::<Result<Vec<_>, _>>()?,
                        )?;
                }

                self.compile(let_recursive.expression(), &variables)?
            }
            ssf::ir::Expression::Let(let_) => {
                let mut variables = variables.clone();

                variables.insert(
                    let_.name().into(),
                    self.compile(let_.bound_expression(), &variables)?,
                );

                self.compile(let_.expression(), &variables)?
            }
            ssf::ir::Expression::Primitive(primitive) => self.compile_primitive(primitive),
            ssf::ir::Expression::PrimitiveOperation(operation) => {
                match (
                    self.compile(operation.lhs(), variables)?,
                    self.compile(operation.rhs(), variables)?,
                ) {
                    (
                        inkwell::values::BasicValueEnum::IntValue(lhs),
                        inkwell::values::BasicValueEnum::IntValue(rhs),
                    ) => match operation.operator() {
                        ssf::ir::PrimitiveOperator::Add => {
                            self.builder.build_int_add(lhs, rhs, "").into()
                        }
                        ssf::ir::PrimitiveOperator::Subtract => {
                            self.builder.build_int_sub(lhs, rhs, "").into()
                        }
                        ssf::ir::PrimitiveOperator::Multiply => {
                            self.builder.build_int_mul(lhs, rhs, "").into()
                        }
                        ssf::ir::PrimitiveOperator::Divide => {
                            self.builder.build_int_signed_div(lhs, rhs, "").into()
                        }
                        ssf::ir::PrimitiveOperator::Equal => self
                            .compile_integer_comparison_operations(
                                inkwell::IntPredicate::EQ,
                                lhs,
                                rhs,
                            ),
                        ssf::ir::PrimitiveOperator::NotEqual => self
                            .compile_integer_comparison_operations(
                                inkwell::IntPredicate::NE,
                                lhs,
                                rhs,
                            ),
                        ssf::ir::PrimitiveOperator::GreaterThan => self
                            .compile_integer_comparison_operations(
                                inkwell::IntPredicate::SGT,
                                lhs,
                                rhs,
                            ),
                        ssf::ir::PrimitiveOperator::GreaterThanOrEqual => self
                            .compile_integer_comparison_operations(
                                inkwell::IntPredicate::SGE,
                                lhs,
                                rhs,
                            ),
                        ssf::ir::PrimitiveOperator::LessThan => self
                            .compile_integer_comparison_operations(
                                inkwell::IntPredicate::SLT,
                                lhs,
                                rhs,
                            ),
                        ssf::ir::PrimitiveOperator::LessThanOrEqual => self
                            .compile_integer_comparison_operations(
                                inkwell::IntPredicate::SLE,
                                lhs,
                                rhs,
                            ),
                    },
                    (
                        inkwell::values::BasicValueEnum::FloatValue(lhs),
                        inkwell::values::BasicValueEnum::FloatValue(rhs),
                    ) => match operation.operator() {
                        ssf::ir::PrimitiveOperator::Add => {
                            self.builder.build_float_add(lhs, rhs, "").into()
                        }
                        ssf::ir::PrimitiveOperator::Subtract => {
                            self.builder.build_float_sub(lhs, rhs, "").into()
                        }
                        ssf::ir::PrimitiveOperator::Multiply => {
                            self.builder.build_float_mul(lhs, rhs, "").into()
                        }
                        ssf::ir::PrimitiveOperator::Divide => {
                            self.builder.build_float_div(lhs, rhs, "").into()
                        }
                        ssf::ir::PrimitiveOperator::Equal => self
                            .compile_float_comparison_operations(
                                inkwell::FloatPredicate::OEQ,
                                lhs,
                                rhs,
                            ),
                        ssf::ir::PrimitiveOperator::NotEqual => self
                            .compile_float_comparison_operations(
                                inkwell::FloatPredicate::ONE,
                                lhs,
                                rhs,
                            ),
                        ssf::ir::PrimitiveOperator::GreaterThan => self
                            .compile_float_comparison_operations(
                                inkwell::FloatPredicate::OGT,
                                lhs,
                                rhs,
                            ),
                        ssf::ir::PrimitiveOperator::GreaterThanOrEqual => self
                            .compile_float_comparison_operations(
                                inkwell::FloatPredicate::OGE,
                                lhs,
                                rhs,
                            ),
                        ssf::ir::PrimitiveOperator::LessThan => self
                            .compile_float_comparison_operations(
                                inkwell::FloatPredicate::OLT,
                                lhs,
                                rhs,
                            ),
                        ssf::ir::PrimitiveOperator::LessThanOrEqual => self
                            .compile_float_comparison_operations(
                                inkwell::FloatPredicate::OLE,
                                lhs,
                                rhs,
                            ),
                    },
                    _ => unreachable!(),
                }
            }
            ssf::ir::Expression::Variable(variable) => {
                self.compile_variable(variable, variables)?
            }
        })
    }

    fn compile_case(
        &self,
        case: &ssf::ir::Case,
        variables: &HashMap<String, inkwell::values::BasicValueEnum<'c>>,
    ) -> Result<inkwell::values::BasicValueEnum<'c>, CompileError> {
        match case {
            ssf::ir::Case::Algebraic(algebraic_case) => {
                let argument = self
                    .compile(algebraic_case.argument(), variables)?
                    .into_struct_value();
                let argument_pointer = self.builder.build_alloca(argument.get_type(), "");
                self.builder.build_store(argument_pointer, argument);

                let tag = if algebraic_case
                    .alternatives()
                    .get(0)
                    .map(|alternative| alternative.constructor().algebraic_type().is_singleton())
                    .unwrap_or(true)
                {
                    // Set a dummy tag value of 0.
                    self.type_compiler.compile_tag().const_int(0, false)
                } else {
                    self.builder
                        .build_extract_value(argument, 0, "")
                        .unwrap()
                        .into_int_value()
                };

                let switch_block = self.builder.get_insert_block().unwrap();
                let phi_block = self.append_basic_block("phi");
                let mut cases = vec![];

                for alternative in algebraic_case.alternatives() {
                    let block = self.append_basic_block("case");
                    self.builder.position_at_end(block);

                    let mut variables = variables.clone();
                    let constructor = alternative.constructor();

                    if !constructor.constructor_type().is_enum() {
                        let argument = self
                            .builder
                            .build_load(
                                self.builder
                                    .build_bitcast(
                                        argument_pointer,
                                        self.type_compiler
                                            .compile_algebraic(
                                                constructor.algebraic_type(),
                                                Some(constructor.tag()),
                                            )
                                            .ptr_type(inkwell::AddressSpace::Generic),
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

                        let constructor_value = if constructor.constructor_type().is_boxed() {
                            self.builder
                                .build_load(constructor_value.into_pointer_value(), "")
                                .into_struct_value()
                        } else {
                            constructor_value.into_struct_value()
                        };

                        for (index, name) in alternative.element_names().iter().enumerate() {
                            variables.insert(
                                name.into(),
                                self.builder
                                    .build_extract_value(constructor_value, index as u32, "")
                                    .unwrap(),
                            );
                        }
                    }

                    let expression = self.compile(alternative.expression(), &variables)?;

                    cases.push((
                        self.type_compiler
                            .compile_tag()
                            .const_int(constructor.tag(), false),
                        block,
                        self.builder.get_insert_block().unwrap(),
                        expression,
                    ));

                    self.builder.build_unconditional_branch(phi_block);
                }

                let mut default_case = None;
                let default_block = self.append_basic_block("default");
                self.builder.position_at_end(default_block);

                if let Some(expression) = algebraic_case.default_alternative() {
                    default_case = Some((
                        self.compile(expression, &variables)?,
                        self.builder.get_insert_block().unwrap(),
                    ));
                    self.builder.build_unconditional_branch(phi_block);
                } else {
                    self.compile_unreachable();
                }

                self.builder.position_at_end(switch_block);
                self.builder.build_switch(
                    tag,
                    default_block,
                    &cases
                        .iter()
                        .map(|(tag, start_block, _, _)| (*tag, *start_block))
                        .collect::<Vec<_>>(),
                );

                self.builder.position_at_end(phi_block);
                let phi = self.builder.build_phi(
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
                            (value as &dyn inkwell::values::BasicValue<'c>, *end_block)
                        })
                        .chain(default_case.as_ref().map(|(value, block)| {
                            (value as &dyn inkwell::values::BasicValue<'c>, *block)
                        }))
                        .collect::<Vec<_>>(),
                );

                Ok(phi.as_basic_value())
            }
            ssf::ir::Case::Primitive(primitive_case) => {
                let argument = self.compile(primitive_case.argument(), variables)?;

                let phi_block = self.append_basic_block("phi");
                let mut cases = vec![];

                for alternative in primitive_case.alternatives() {
                    let then_block = self.append_basic_block("then");
                    let else_block = self.append_basic_block("else");

                    self.builder.build_conditional_branch(
                        if argument.is_int_value() {
                            self.builder.build_int_compare(
                                inkwell::IntPredicate::EQ,
                                argument.into_int_value(),
                                self.compile_primitive(alternative.primitive())
                                    .into_int_value(),
                                "",
                            )
                        } else {
                            self.builder.build_float_compare(
                                inkwell::FloatPredicate::OEQ,
                                argument.into_float_value(),
                                self.compile_primitive(alternative.primitive())
                                    .into_float_value(),
                                "",
                            )
                        },
                        then_block,
                        else_block,
                    );
                    self.builder.position_at_end(then_block);

                    cases.push((
                        self.compile(alternative.expression(), &variables)?,
                        self.builder.get_insert_block().unwrap(),
                    ));

                    self.builder.build_unconditional_branch(phi_block);
                    self.builder.position_at_end(else_block);
                }

                if let Some(expression) = primitive_case.default_alternative() {
                    cases.push((
                        self.compile(expression, &variables)?,
                        self.builder.get_insert_block().unwrap(),
                    ));
                    self.builder.build_unconditional_branch(phi_block);
                } else {
                    self.compile_unreachable();
                }

                self.builder.position_at_end(phi_block);
                let phi = self.builder.build_phi(cases[0].0.get_type(), "");
                phi.add_incoming(
                    &cases
                        .iter()
                        .map(|(value, block)| {
                            (value as &dyn inkwell::values::BasicValue<'c>, *block)
                        })
                        .collect::<Vec<_>>(),
                );

                Ok(phi.as_basic_value())
            }
        }
    }

    fn compile_integer_comparison_operations(
        &self,
        predicate: inkwell::IntPredicate,
        lhs: inkwell::values::IntValue<'c>,
        rhs: inkwell::values::IntValue<'c>,
    ) -> inkwell::values::BasicValueEnum<'c> {
        self.builder.build_cast(
            inkwell::values::InstructionOpcode::ZExt,
            self.builder.build_int_compare(predicate, lhs, rhs, ""),
            self.type_compiler
                .compile_primitive(&ssf::types::Primitive::Integer8),
            "",
        )
    }

    fn compile_float_comparison_operations(
        &self,
        predicate: inkwell::FloatPredicate,
        lhs: inkwell::values::FloatValue<'c>,
        rhs: inkwell::values::FloatValue<'c>,
    ) -> inkwell::values::BasicValueEnum<'c> {
        self.builder.build_cast(
            inkwell::values::InstructionOpcode::ZExt,
            self.builder.build_float_compare(predicate, lhs, rhs, ""),
            self.type_compiler
                .compile_primitive(&ssf::types::Primitive::Integer8),
            "",
        )
    }

    fn compile_primitive(
        &self,
        primitive: &ssf::ir::Primitive,
    ) -> inkwell::values::BasicValueEnum<'c> {
        match primitive {
            ssf::ir::Primitive::Float32(number) => {
                self.context.f32_type().const_float(*number as f64).into()
            }
            ssf::ir::Primitive::Float64(number) => {
                self.context.f64_type().const_float(*number).into()
            }
            ssf::ir::Primitive::Integer8(number) => self
                .context
                .i8_type()
                .const_int(*number as u64, false)
                .into(),
            ssf::ir::Primitive::Integer32(number) => self
                .context
                .i32_type()
                .const_int(*number as u64, false)
                .into(),
            ssf::ir::Primitive::Integer64(number) => {
                self.context.i64_type().const_int(*number, false).into()
            }
        }
    }

    fn compile_variable(
        &self,
        variable: &ssf::ir::Variable,
        variables: &HashMap<String, inkwell::values::BasicValueEnum<'c>>,
    ) -> Result<inkwell::values::BasicValueEnum<'c>, CompileError> {
        variables
            .get(variable.name())
            .copied()
            .ok_or_else(|| CompileError::VariableNotFound(variable.name().into()))
    }

    fn append_basic_block(&self, name: &str) -> inkwell::basic_block::BasicBlock<'c> {
        self.context.append_basic_block(
            self.builder
                .get_insert_block()
                .unwrap()
                .get_parent()
                .unwrap(),
            name,
        )
    }

    fn compile_malloc(
        &self,
        type_: inkwell::types::StructType<'c>,
    ) -> inkwell::values::PointerValue<'c> {
        self.malloc_compiler
            .compile_struct_malloc(&self.builder, type_)
    }

    fn compile_unreachable(&self) {
        self.builder.build_unreachable();
    }

    fn is_bitcast_supported(&self, type_: inkwell::types::BasicTypeEnum<'c>) -> bool {
        type_.is_int_type() || type_.is_float_type() || type_.is_pointer_type()
    }
}

#[cfg(test)]
mod tests {
    use super::super::compile_configuration::COMPILE_CONFIGURATION;
    use super::super::expression_compiler_factory::ExpressionCompilerFactory;
    use super::*;

    fn create_expression_compiler(
        context: &inkwell::context::Context,
    ) -> (
        Arc<ExpressionCompiler>,
        Arc<TypeCompiler>,
        Arc<inkwell::builder::Builder>,
        Arc<inkwell::module::Module>,
        inkwell::values::FunctionValue,
    ) {
        let module = Arc::new(context.create_module(""));

        module.add_function(
            &COMPILE_CONFIGURATION.malloc_function_name,
            context
                .i8_type()
                .ptr_type(inkwell::AddressSpace::Generic)
                .fn_type(&[context.i64_type().into()], false),
            None,
        );

        let function = module.add_function("", context.void_type().fn_type(&[], false), None);
        let builder = Arc::new(context.create_builder());
        builder.position_at_end(context.append_basic_block(function, "entry"));

        let type_compiler = TypeCompiler::new(&context);
        let closure_operation_compiler =
            ClosureOperationCompiler::new(context, type_compiler.clone());
        let malloc_compiler = MallocCompiler::new(module.clone(), COMPILE_CONFIGURATION.clone());
        let function_application_compiler = FunctionApplicationCompiler::new(
            &context,
            module.clone(),
            type_compiler.clone(),
            closure_operation_compiler.clone(),
            malloc_compiler.clone(),
        );
        let expression_compiler_factory = ExpressionCompilerFactory::new(
            context,
            function_application_compiler.clone(),
            type_compiler.clone(),
            closure_operation_compiler.clone(),
            malloc_compiler.clone(),
        );

        (
            expression_compiler_factory.create(
                builder.clone(),
                FunctionCompiler::new(
                    &context,
                    module.clone(),
                    expression_compiler_factory.clone(),
                    type_compiler.clone(),
                    HashMap::new().into(),
                ),
            ),
            type_compiler,
            builder,
            module,
            function,
        )
    }

    mod case_expressions {
        use super::*;

        mod algebraic {
            use super::*;

            #[test]
            fn compile_algebraic_case_expression_with_multiple_constructors() {
                let algebraic_type = ssf::types::Algebraic::new(vec![
                    ssf::types::Constructor::boxed(vec![]),
                    ssf::types::Constructor::boxed(vec![ssf::types::Primitive::Float64.into()]),
                ]);

                for algebraic_case in vec![
                    ssf::ir::AlgebraicCase::new(
                        ssf::ir::Variable::new("x"),
                        vec![],
                        Some(42.0.into()),
                    ),
                    ssf::ir::AlgebraicCase::new(
                        ssf::ir::Variable::new("x"),
                        vec![ssf::ir::AlgebraicAlternative::new(
                            ssf::ir::Constructor::new(algebraic_type.clone(), 0),
                            vec![],
                            42.0,
                        )],
                        None,
                    ),
                    ssf::ir::AlgebraicCase::new(
                        ssf::ir::Variable::new("x"),
                        vec![ssf::ir::AlgebraicAlternative::new(
                            ssf::ir::Constructor::new(algebraic_type.clone(), 1),
                            vec!["y".into()],
                            ssf::ir::Variable::new("y"),
                        )],
                        None,
                    ),
                    ssf::ir::AlgebraicCase::new(
                        ssf::ir::Variable::new("x"),
                        vec![
                            ssf::ir::AlgebraicAlternative::new(
                                ssf::ir::Constructor::new(algebraic_type.clone(), 0),
                                vec![],
                                42.0,
                            ),
                            ssf::ir::AlgebraicAlternative::new(
                                ssf::ir::Constructor::new(algebraic_type.clone(), 1),
                                vec!["y".into()],
                                ssf::ir::Variable::new("y"),
                            ),
                        ],
                        None,
                    ),
                    ssf::ir::AlgebraicCase::new(
                        ssf::ir::Variable::new("x"),
                        vec![
                            ssf::ir::AlgebraicAlternative::new(
                                ssf::ir::Constructor::new(algebraic_type.clone(), 0),
                                vec![],
                                42.0,
                            ),
                            ssf::ir::AlgebraicAlternative::new(
                                ssf::ir::Constructor::new(algebraic_type.clone(), 1),
                                vec!["y".into()],
                                ssf::ir::Variable::new("y"),
                            ),
                        ],
                        Some(42.0.into()),
                    ),
                    ssf::ir::AlgebraicCase::new(
                        ssf::ir::Variable::new("x"),
                        vec![ssf::ir::AlgebraicAlternative::new(
                            ssf::ir::Constructor::new(algebraic_type.clone(), 0),
                            vec![],
                            42.0,
                        )],
                        Some(
                            ssf::ir::AlgebraicCase::new(
                                ssf::ir::Variable::new("x"),
                                vec![ssf::ir::AlgebraicAlternative::new(
                                    ssf::ir::Constructor::new(algebraic_type.clone(), 0),
                                    vec![],
                                    42.0,
                                )],
                                Some(42.0.into()),
                            )
                            .into(),
                        ),
                    ),
                ] {
                    let context = inkwell::context::Context::create();
                    let (expression_compiler, type_compiler, builder, module, function) =
                        create_expression_compiler(&context);

                    expression_compiler
                        .compile(
                            &algebraic_case.into(),
                            &vec![(
                                "x".into(),
                                type_compiler
                                    .compile(&algebraic_type.clone().into())
                                    .into_struct_type()
                                    .get_undef()
                                    .into(),
                            )]
                            .drain(..)
                            .collect(),
                        )
                        .unwrap();

                    builder.build_return(None);

                    assert!(function.verify(true));
                    assert!(module.verify().is_ok());
                }
            }

            #[test]
            fn compile_algebraic_case_expression_with_single_constructors() {
                let algebraic_type =
                    ssf::types::Algebraic::new(vec![ssf::types::Constructor::boxed(vec![])]);

                for algebraic_case in vec![
                    ssf::ir::AlgebraicCase::new(
                        ssf::ir::Variable::new("x"),
                        vec![],
                        Some(42.0.into()),
                    ),
                    ssf::ir::AlgebraicCase::new(
                        ssf::ir::Variable::new("x"),
                        vec![ssf::ir::AlgebraicAlternative::new(
                            ssf::ir::Constructor::new(algebraic_type.clone(), 0),
                            vec![],
                            42.0,
                        )],
                        None,
                    ),
                    ssf::ir::AlgebraicCase::new(
                        ssf::ir::Variable::new("x"),
                        vec![ssf::ir::AlgebraicAlternative::new(
                            ssf::ir::Constructor::new(algebraic_type.clone(), 0),
                            vec![],
                            42.0,
                        )],
                        Some(42.0.into()),
                    ),
                ] {
                    let context = inkwell::context::Context::create();
                    let (expression_compiler, type_compiler, builder, module, function) =
                        create_expression_compiler(&context);

                    expression_compiler
                        .compile(
                            &algebraic_case.into(),
                            &vec![(
                                "x".into(),
                                type_compiler
                                    .compile(&algebraic_type.clone().into())
                                    .into_struct_type()
                                    .get_undef()
                                    .into(),
                            )]
                            .drain(..)
                            .collect(),
                        )
                        .unwrap();

                    builder.build_return(None);

                    assert!(function.verify(true));
                    assert!(module.verify().is_ok());
                }
            }

            #[test]
            fn compile_algebraic_case_expression_with_unboxed_constructors() {
                let algebraic_type = ssf::types::Algebraic::new(vec![
                    ssf::types::Constructor::unboxed(vec![]),
                    ssf::types::Constructor::unboxed(vec![ssf::types::Primitive::Float64.into()]),
                ]);

                for algebraic_case in vec![
                    ssf::ir::AlgebraicCase::new(
                        ssf::ir::Variable::new("x"),
                        vec![],
                        Some(42.0.into()),
                    ),
                    ssf::ir::AlgebraicCase::new(
                        ssf::ir::Variable::new("x"),
                        vec![ssf::ir::AlgebraicAlternative::new(
                            ssf::ir::Constructor::new(algebraic_type.clone(), 0),
                            vec![],
                            42.0,
                        )],
                        None,
                    ),
                    ssf::ir::AlgebraicCase::new(
                        ssf::ir::Variable::new("x"),
                        vec![ssf::ir::AlgebraicAlternative::new(
                            ssf::ir::Constructor::new(algebraic_type.clone(), 1),
                            vec!["y".into()],
                            ssf::ir::Variable::new("y"),
                        )],
                        None,
                    ),
                    ssf::ir::AlgebraicCase::new(
                        ssf::ir::Variable::new("x"),
                        vec![
                            ssf::ir::AlgebraicAlternative::new(
                                ssf::ir::Constructor::new(algebraic_type.clone(), 0),
                                vec![],
                                42.0,
                            ),
                            ssf::ir::AlgebraicAlternative::new(
                                ssf::ir::Constructor::new(algebraic_type.clone(), 1),
                                vec!["y".into()],
                                ssf::ir::Variable::new("y"),
                            ),
                        ],
                        None,
                    ),
                    ssf::ir::AlgebraicCase::new(
                        ssf::ir::Variable::new("x"),
                        vec![
                            ssf::ir::AlgebraicAlternative::new(
                                ssf::ir::Constructor::new(algebraic_type.clone(), 0),
                                vec![],
                                42.0,
                            ),
                            ssf::ir::AlgebraicAlternative::new(
                                ssf::ir::Constructor::new(algebraic_type.clone(), 1),
                                vec!["y".into()],
                                ssf::ir::Variable::new("y"),
                            ),
                        ],
                        Some(42.0.into()),
                    ),
                ] {
                    let context = inkwell::context::Context::create();
                    let (expression_compiler, type_compiler, builder, module, function) =
                        create_expression_compiler(&context);

                    expression_compiler
                        .compile(
                            &algebraic_case.into(),
                            &vec![(
                                "x".into(),
                                type_compiler
                                    .compile(&algebraic_type.clone().into())
                                    .into_struct_type()
                                    .get_undef()
                                    .into(),
                            )]
                            .drain(..)
                            .collect(),
                        )
                        .unwrap();

                    builder.build_return(None);

                    assert!(function.verify(true));
                    assert!(module.verify().is_ok());
                }
            }
        }

        mod primitive {
            use super::*;

            #[test]
            fn compile_integer_case_expression() {
                for primitive_case in vec![
                    ssf::ir::PrimitiveCase::new(
                        ssf::ir::Primitive::Integer64(42),
                        vec![],
                        Some(42.into()),
                    ),
                    ssf::ir::PrimitiveCase::new(
                        ssf::ir::Primitive::Integer64(42),
                        vec![ssf::ir::PrimitiveAlternative::new(0, 42)],
                        None,
                    ),
                    ssf::ir::PrimitiveCase::new(
                        ssf::ir::Primitive::Integer64(42),
                        vec![ssf::ir::PrimitiveAlternative::new(0, 42)],
                        Some(42.into()),
                    ),
                    ssf::ir::PrimitiveCase::new(
                        ssf::ir::Primitive::Integer64(42),
                        vec![
                            ssf::ir::PrimitiveAlternative::new(0, 42),
                            ssf::ir::PrimitiveAlternative::new(1, 42),
                        ],
                        None,
                    ),
                    ssf::ir::PrimitiveCase::new(
                        ssf::ir::Primitive::Integer64(42),
                        vec![
                            ssf::ir::PrimitiveAlternative::new(0, 42),
                            ssf::ir::PrimitiveAlternative::new(1, 42),
                        ],
                        Some(42.into()),
                    ),
                ] {
                    let context = inkwell::context::Context::create();
                    let (expression_compiler, _, builder, module, function) =
                        create_expression_compiler(&context);

                    expression_compiler
                        .compile(&primitive_case.into(), &Default::default())
                        .unwrap();

                    builder.build_return(None);

                    assert!(function.verify(true));
                    assert!(module.verify().is_ok());
                }
            }

            #[test]
            fn compile_float_case_expression() {
                for primitive_case in vec![
                    ssf::ir::PrimitiveCase::new(
                        ssf::ir::Primitive::Float64(42.0),
                        vec![],
                        Some(42.0.into()),
                    ),
                    ssf::ir::PrimitiveCase::new(
                        ssf::ir::Primitive::Float64(42.0),
                        vec![ssf::ir::PrimitiveAlternative::new(0.0, 42.0)],
                        None,
                    ),
                    ssf::ir::PrimitiveCase::new(
                        ssf::ir::Primitive::Float64(42.0),
                        vec![ssf::ir::PrimitiveAlternative::new(0.0, 42.0)],
                        Some(42.0.into()),
                    ),
                    ssf::ir::PrimitiveCase::new(
                        ssf::ir::Primitive::Float64(42.0),
                        vec![
                            ssf::ir::PrimitiveAlternative::new(0.0, 42.0),
                            ssf::ir::PrimitiveAlternative::new(1.0, 42.0),
                        ],
                        None,
                    ),
                    ssf::ir::PrimitiveCase::new(
                        ssf::ir::Primitive::Float64(42.0),
                        vec![
                            ssf::ir::PrimitiveAlternative::new(0.0, 42.0),
                            ssf::ir::PrimitiveAlternative::new(1.0, 42.0),
                        ],
                        Some(42.0.into()),
                    ),
                    ssf::ir::PrimitiveCase::new(
                        ssf::ir::Primitive::Float64(42.0),
                        vec![ssf::ir::PrimitiveAlternative::new(0.0, 42.0)],
                        Some(
                            ssf::ir::PrimitiveCase::new(
                                ssf::ir::Primitive::Float64(42.0),
                                vec![ssf::ir::PrimitiveAlternative::new(0.0, 42.0)],
                                Some(42.0.into()),
                            )
                            .into(),
                        ),
                    ),
                ] {
                    let context = inkwell::context::Context::create();
                    let (expression_compiler, _, builder, module, function) =
                        create_expression_compiler(&context);

                    expression_compiler
                        .compile(&primitive_case.into(), &Default::default())
                        .unwrap();

                    builder.build_return(None);

                    assert!(function.verify(true));
                    assert!(module.verify().is_ok());
                }
            }
        }
    }

    mod constructor_applications {
        use super::*;

        #[test]
        fn compile_boxed_constructor_applications() {
            let algebraic_type = ssf::types::Algebraic::new(vec![
                ssf::types::Constructor::boxed(vec![]),
                ssf::types::Constructor::boxed(vec![ssf::types::Primitive::Float64.into()]),
            ]);

            for constructor_application in vec![
                ssf::ir::ConstructorApplication::new(
                    ssf::ir::Constructor::new(algebraic_type.clone(), 0),
                    vec![],
                ),
                ssf::ir::ConstructorApplication::new(
                    ssf::ir::Constructor::new(algebraic_type, 1),
                    vec![42.0.into()],
                ),
            ] {
                let context = inkwell::context::Context::create();
                let (expression_compiler, _, builder, module, function) =
                    create_expression_compiler(&context);

                expression_compiler
                    .compile(&constructor_application.into(), &HashMap::new())
                    .unwrap();

                builder.build_return(None);

                assert!(function.verify(true));
                assert!(module.verify().is_ok());
            }
        }

        #[test]
        fn compile_unboxed_constructor_applications() {
            let algebraic_type = ssf::types::Algebraic::new(vec![
                ssf::types::Constructor::unboxed(vec![]),
                ssf::types::Constructor::unboxed(vec![ssf::types::Primitive::Float64.into()]),
            ]);

            for constructor_application in vec![
                ssf::ir::ConstructorApplication::new(
                    ssf::ir::Constructor::new(algebraic_type.clone(), 0),
                    vec![],
                ),
                ssf::ir::ConstructorApplication::new(
                    ssf::ir::Constructor::new(algebraic_type, 1),
                    vec![42.0.into()],
                ),
            ] {
                let context = inkwell::context::Context::create();
                let (expression_compiler, _, builder, module, function) =
                    create_expression_compiler(&context);

                expression_compiler
                    .compile(&constructor_application.into(), &HashMap::new())
                    .unwrap();

                builder.build_return(None);

                assert!(function.verify(true));
                assert!(module.verify().is_ok());
            }
        }
    }
}
