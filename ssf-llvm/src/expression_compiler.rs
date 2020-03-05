use super::compile_configuration::CompileConfiguration;
use super::error::CompileError;
use super::function_compiler::FunctionCompiler;
use super::type_compiler::TypeCompiler;
use std::collections::HashMap;

pub struct ExpressionCompiler<'c, 'm, 'b, 'f, 't, 'v> {
    context: &'c inkwell::context::Context,
    module: &'m inkwell::module::Module<'c>,
    builder: &'b inkwell::builder::Builder<'c>,
    function_compiler: &'f FunctionCompiler<'c, 'm, 't, 'v>,
    type_compiler: &'t TypeCompiler<'c>,
    compile_configuration: &'m CompileConfiguration,
}

impl<'c, 'm, 'b, 'f, 't, 'v> ExpressionCompiler<'c, 'm, 'b, 'f, 't, 'v> {
    pub fn new(
        context: &'c inkwell::context::Context,
        module: &'m inkwell::module::Module<'c>,
        builder: &'b inkwell::builder::Builder<'c>,
        function_compiler: &'f FunctionCompiler<'c, 'm, 't, 'v>,
        type_compiler: &'t TypeCompiler<'c>,
        compile_configuration: &'m CompileConfiguration,
    ) -> Self {
        Self {
            context,
            module,
            builder,
            function_compiler,
            type_compiler,
            compile_configuration,
        }
    }

    pub fn compile(
        &self,
        expression: &ssf::ir::Expression,
        variables: &HashMap<String, inkwell::values::BasicValueEnum<'c>>,
    ) -> Result<inkwell::values::BasicValueEnum<'c>, CompileError> {
        match expression {
            ssf::ir::Expression::Case(case) => self.compile_case(case, variables),
            ssf::ir::Expression::ConstructorApplication(constructor_application) => {
                let algebraic_type = constructor_application.constructor().algebraic_type();
                let constructor_type = constructor_application.constructor().constructor_type();

                let mut algebraic_value = self
                    .type_compiler
                    .compile_algebraic(
                        algebraic_type,
                        Some(constructor_application.constructor().index()),
                    )
                    .const_zero()
                    .into();

                if !algebraic_type.is_singleton() {
                    algebraic_value = self
                        .builder
                        .build_insert_value(
                            algebraic_value,
                            self.context.i64_type().const_int(
                                constructor_application.constructor().index() as u64,
                                false,
                            ),
                            0,
                            "",
                        )
                        .unwrap();
                }

                if !constructor_type.is_enum() {
                    let constructor_type = self
                        .type_compiler
                        .compile_unboxed_constructor(constructor_type);

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
                        if constructor_application
                            .constructor()
                            .constructor_type()
                            .is_boxed()
                        {
                            let constructor_pointer = self.compile_struct_malloc(constructor_type);

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

                Ok(self.builder.build_load(algebraic_pointer, ""))
            }
            ssf::ir::Expression::FunctionApplication(function_application) => {
                let closure = self
                    .compile_variable(function_application.function(), variables)?
                    .into_pointer_value();

                let mut arguments = vec![unsafe {
                    self.builder.build_gep(
                        closure,
                        &[
                            self.context.i32_type().const_int(0, false),
                            self.context.i32_type().const_int(1, false),
                        ],
                        "",
                    )
                }
                .into()];

                for argument in function_application.arguments() {
                    arguments.push(self.compile(argument, variables)?);
                }

                Ok(self
                    .builder
                    .build_call(
                        self.builder
                            .build_load(
                                unsafe {
                                    self.builder.build_gep(
                                        closure,
                                        &[
                                            self.context.i32_type().const_int(0, false),
                                            self.context.i32_type().const_int(0, false),
                                        ],
                                        "",
                                    )
                                },
                                "",
                            )
                            .into_pointer_value(),
                        &arguments,
                        "",
                    )
                    .try_as_basic_value()
                    .left()
                    .unwrap())
            }
            ssf::ir::Expression::LetFunctions(let_functions) => {
                let mut variables = variables.clone();
                let mut closures = HashMap::<&str, inkwell::values::PointerValue>::new();

                for definition in let_functions.definitions() {
                    let closure_type = self.type_compiler.compile_closure(definition);
                    let pointer = self.compile_struct_malloc(closure_type);

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

                for definition in let_functions.definitions() {
                    let closure = closures[definition.name()];

                    self.builder.build_store(
                        unsafe {
                            self.builder.build_gep(
                                closure,
                                &[
                                    self.context.i32_type().const_int(0, false),
                                    self.context.i32_type().const_int(0, false),
                                ],
                                "",
                            )
                        },
                        self.function_compiler
                            .compile(definition)?
                            .as_global_value()
                            .as_pointer_value(),
                    );

                    for (index, value) in definition
                        .environment()
                        .iter()
                        .map(|argument| variables.get(argument.name()).copied())
                        .collect::<Option<Vec<_>>>()
                        .ok_or(CompileError::VariableNotFound)?
                        .iter()
                        .enumerate()
                    {
                        self.builder.build_store(
                            unsafe {
                                self.builder.build_gep(
                                    closure,
                                    &[
                                        self.context.i32_type().const_int(0, false),
                                        self.context.i32_type().const_int(1, false),
                                        self.context.i32_type().const_int(index as u64, false),
                                    ],
                                    "",
                                )
                            },
                            *value,
                        );
                    }
                }

                self.compile(let_functions.expression(), &variables)
            }
            ssf::ir::Expression::LetValues(let_values) => {
                let mut variables = variables.clone();

                for definition in let_values.definitions() {
                    variables.insert(
                        definition.name().into(),
                        self.compile(definition.body(), &variables)?,
                    );
                }

                self.compile(let_values.expression(), &variables)
            }
            ssf::ir::Expression::Primitive(primitive) => Ok(self.compile_primitive(primitive)),
            ssf::ir::Expression::Operation(operation) => {
                let lhs = self.compile(operation.lhs(), variables)?.into_float_value();
                let rhs = self.compile(operation.rhs(), variables)?.into_float_value();

                Ok(match operation.operator() {
                    ssf::ir::Operator::Add => self.builder.build_float_add(lhs, rhs, ""),
                    ssf::ir::Operator::Subtract => self.builder.build_float_sub(lhs, rhs, ""),
                    ssf::ir::Operator::Multiply => self.builder.build_float_mul(lhs, rhs, ""),
                    ssf::ir::Operator::Divide => self.builder.build_float_div(lhs, rhs, ""),
                }
                .into())
            }
            ssf::ir::Expression::Variable(variable) => self.compile_variable(variable, variables),
        }
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

                let tag = if algebraic_case.alternatives().is_empty()
                    || algebraic_case.alternatives()[0]
                        .constructor()
                        .algebraic_type()
                        .is_singleton()
                {
                    self.context.i64_type().const_int(0, false)
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
                                                Some(constructor.index()),
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
                                if alternative.constructor().algebraic_type().is_singleton() {
                                    0
                                } else {
                                    1
                                },
                                "",
                            )
                            .unwrap();

                        let constructor_value =
                            if alternative.constructor().constructor_type().is_boxed() {
                                self.builder
                                    .build_load(
                                        self.builder
                                            .build_bitcast(
                                                constructor_value,
                                                self.type_compiler.compile_constructor(
                                                    alternative.constructor().constructor_type(),
                                                ),
                                                "",
                                            )
                                            .into_pointer_value(),
                                        "",
                                    )
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

                    cases.push((
                        self.context
                            .i64_type()
                            .const_int(alternative.constructor().index() as u64, false),
                        block,
                        self.compile(alternative.expression(), &variables)?,
                    ));

                    self.builder.build_unconditional_branch(phi_block);
                }

                let mut default_value = None;
                let default_block = self.append_basic_block("default");
                self.builder.position_at_end(default_block);

                if let Some(default_alternative) = algebraic_case.default_alternative() {
                    let mut variables = variables.clone();

                    variables.insert(default_alternative.variable().into(), argument.into());

                    default_value =
                        Some(self.compile(default_alternative.expression(), &variables)?);
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
                        .map(|(tag, block, _)| (*tag, *block))
                        .collect::<Vec<_>>(),
                );

                self.builder.position_at_end(phi_block);
                let phi = self.builder.build_phi(
                    cases
                        .get(0)
                        .map(|(_, _, value)| value.get_type())
                        .unwrap_or_else(|| default_value.unwrap().get_type()),
                    "",
                );
                phi.add_incoming(
                    &cases
                        .iter()
                        .map(|(_, block, value)| {
                            (value as &dyn inkwell::values::BasicValue<'c>, *block)
                        })
                        .chain(match &default_value {
                            Some(default_value) => vec![(
                                default_value as &dyn inkwell::values::BasicValue<'c>,
                                default_block,
                            )],
                            None => vec![],
                        })
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
                        then_block,
                    ));

                    self.builder.build_unconditional_branch(phi_block);
                    self.builder.position_at_end(else_block);
                }

                if let Some(default_alternative) = primitive_case.default_alternative() {
                    let mut variables = variables.clone();

                    variables.insert(default_alternative.variable().into(), argument);

                    cases.push((
                        self.compile(default_alternative.expression(), &variables)?,
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

    fn compile_primitive(
        &self,
        primitive: &ssf::ir::Primitive,
    ) -> inkwell::values::BasicValueEnum<'c> {
        match primitive {
            ssf::ir::Primitive::Float64(number) => {
                self.context.f64_type().const_float(*number).into()
            }
            ssf::ir::Primitive::Integer8(number) => self
                .context
                .i8_type()
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
        match variables.get(variable.name()) {
            Some(value) => Ok(self.unwrap_value(*value)),
            None => Err(CompileError::VariableNotFound),
        }
    }

    fn unwrap_value(
        &self,
        value: inkwell::values::BasicValueEnum<'c>,
    ) -> inkwell::values::BasicValueEnum<'c> {
        match value {
            inkwell::values::BasicValueEnum::PointerValue(value) => {
                match value.get_type().get_element_type() {
                    inkwell::types::AnyTypeEnum::FloatType(_) => self.builder.build_load(value, ""),
                    _ => value.into(),
                }
            }
            _ => value,
        }
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

    fn compile_struct_malloc(
        &self,
        type_: inkwell::types::StructType<'c>,
    ) -> inkwell::values::PointerValue<'c> {
        self.builder
            .build_bitcast(
                self.builder
                    .build_call(
                        self.module
                            .get_function(self.compile_configuration.malloc_function_name())
                            .unwrap(),
                        &[type_.size_of().unwrap().into()],
                        "",
                    )
                    .try_as_basic_value()
                    .left()
                    .unwrap(),
                type_.ptr_type(inkwell::AddressSpace::Generic),
                "",
            )
            .into_pointer_value()
    }

    fn compile_unreachable(&self) {
        if let Some(panic_function_name) = self.compile_configuration.panic_function_name() {
            self.builder.build_call(
                self.module.get_function(panic_function_name).unwrap(),
                &[],
                "",
            );
        }

        self.builder.build_unreachable();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod case_expressions {
        use super::*;

        mod algebraic {
            use super::*;

            #[test]
            fn compile_algebraic_case_expression_with_multiple_constructors() {
                let compile_configuration = CompileConfiguration::new("", vec![], None, None);
                let algebraic_type = ssf::types::Algebraic::new(vec![
                    ssf::types::Constructor::boxed(vec![]),
                    ssf::types::Constructor::boxed(vec![ssf::types::Primitive::Float64.into()]),
                ]);

                for algebraic_case in vec![
                    ssf::ir::AlgebraicCase::new(
                        ssf::ir::Variable::new("x"),
                        vec![],
                        Some(ssf::ir::DefaultAlternative::new("x", 42.0)),
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
                        Some(ssf::ir::DefaultAlternative::new("x", 42.0)),
                    ),
                ] {
                    let context = inkwell::context::Context::create();
                    let type_compiler = TypeCompiler::new(&context);
                    let module = context.create_module("");
                    let function =
                        module.add_function("", context.void_type().fn_type(&[], false), None);
                    let builder = context.create_builder();
                    builder.position_at_end(context.append_basic_block(function, "entry"));

                    ExpressionCompiler::new(
                        &context,
                        &module,
                        &builder,
                        &FunctionCompiler::new(
                            &context,
                            &module,
                            &type_compiler,
                            &HashMap::new(),
                            &compile_configuration,
                        ),
                        &type_compiler,
                        &compile_configuration,
                    )
                    .compile(
                        &algebraic_case.into(),
                        &vec![(
                            "x".into(),
                            type_compiler
                                .compile_value(&algebraic_type.clone().into())
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
                let compile_configuration = CompileConfiguration::new("", vec![], None, None);
                let algebraic_type =
                    ssf::types::Algebraic::new(vec![ssf::types::Constructor::boxed(vec![])]);

                for algebraic_case in vec![
                    ssf::ir::AlgebraicCase::new(
                        ssf::ir::Variable::new("x"),
                        vec![],
                        Some(ssf::ir::DefaultAlternative::new("x", 42.0)),
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
                        Some(ssf::ir::DefaultAlternative::new("x", 42.0)),
                    ),
                ] {
                    let context = inkwell::context::Context::create();
                    let type_compiler = TypeCompiler::new(&context);
                    let module = context.create_module("");
                    let function =
                        module.add_function("", context.void_type().fn_type(&[], false), None);
                    let builder = context.create_builder();
                    builder.position_at_end(context.append_basic_block(function, "entry"));

                    ExpressionCompiler::new(
                        &context,
                        &module,
                        &builder,
                        &FunctionCompiler::new(
                            &context,
                            &module,
                            &type_compiler,
                            &HashMap::new(),
                            &compile_configuration,
                        ),
                        &type_compiler,
                        &compile_configuration,
                    )
                    .compile(
                        &algebraic_case.into(),
                        &vec![(
                            "x".into(),
                            type_compiler
                                .compile_value(&algebraic_type.clone().into())
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
                let compile_configuration = CompileConfiguration::new("", vec![], None, None);

                for primitive_case in vec![
                    ssf::ir::PrimitiveCase::new(
                        ssf::ir::Variable::new("x"),
                        vec![],
                        Some(ssf::ir::DefaultAlternative::new("x", 42)),
                    ),
                    ssf::ir::PrimitiveCase::new(
                        ssf::ir::Variable::new("x"),
                        vec![ssf::ir::PrimitiveAlternative::new(0, 42)],
                        None,
                    ),
                    ssf::ir::PrimitiveCase::new(
                        ssf::ir::Variable::new("x"),
                        vec![ssf::ir::PrimitiveAlternative::new(0, 42)],
                        Some(ssf::ir::DefaultAlternative::new("x", 42)),
                    ),
                    ssf::ir::PrimitiveCase::new(
                        ssf::ir::Variable::new("x"),
                        vec![
                            ssf::ir::PrimitiveAlternative::new(0, 42),
                            ssf::ir::PrimitiveAlternative::new(1, 42),
                        ],
                        None,
                    ),
                    ssf::ir::PrimitiveCase::new(
                        ssf::ir::Variable::new("x"),
                        vec![
                            ssf::ir::PrimitiveAlternative::new(0, 42),
                            ssf::ir::PrimitiveAlternative::new(1, 42),
                        ],
                        Some(ssf::ir::DefaultAlternative::new("x", 42)),
                    ),
                ] {
                    let context = inkwell::context::Context::create();
                    let type_compiler = TypeCompiler::new(&context);
                    let module = context.create_module("");
                    let function = module.add_function(
                        "",
                        context.i64_type().fn_type(
                            &[type_compiler
                                .compile_value(&ssf::types::Primitive::Integer64.into())],
                            false,
                        ),
                        None,
                    );
                    let builder = context.create_builder();
                    builder.position_at_end(context.append_basic_block(function, "entry"));

                    builder.build_return(Some(
                        &ExpressionCompiler::new(
                            &context,
                            &module,
                            &builder,
                            &FunctionCompiler::new(
                                &context,
                                &module,
                                &type_compiler,
                                &HashMap::new(),
                                &compile_configuration,
                            ),
                            &type_compiler,
                            &compile_configuration,
                        )
                        .compile(
                            &primitive_case.into(),
                            &vec![("x".into(), function.get_params()[0])]
                                .drain(..)
                                .collect(),
                        )
                        .unwrap(),
                    ));

                    assert!(function.verify(true));
                    assert!(module.verify().is_ok());
                }
            }

            #[test]
            fn compile_float_case_expression() {
                let compile_configuration = CompileConfiguration::new("", vec![], None, None);

                for primitive_case in vec![
                    ssf::ir::PrimitiveCase::new(
                        ssf::ir::Variable::new("x"),
                        vec![],
                        Some(ssf::ir::DefaultAlternative::new("x", 42.0)),
                    ),
                    ssf::ir::PrimitiveCase::new(
                        ssf::ir::Variable::new("x"),
                        vec![ssf::ir::PrimitiveAlternative::new(0.0, 42.0)],
                        None,
                    ),
                    ssf::ir::PrimitiveCase::new(
                        ssf::ir::Variable::new("x"),
                        vec![ssf::ir::PrimitiveAlternative::new(0.0, 42.0)],
                        Some(ssf::ir::DefaultAlternative::new("x", 42.0)),
                    ),
                    ssf::ir::PrimitiveCase::new(
                        ssf::ir::Variable::new("x"),
                        vec![
                            ssf::ir::PrimitiveAlternative::new(0.0, 42.0),
                            ssf::ir::PrimitiveAlternative::new(1.0, 42.0),
                        ],
                        None,
                    ),
                    ssf::ir::PrimitiveCase::new(
                        ssf::ir::Variable::new("x"),
                        vec![
                            ssf::ir::PrimitiveAlternative::new(0.0, 42.0),
                            ssf::ir::PrimitiveAlternative::new(1.0, 42.0),
                        ],
                        Some(ssf::ir::DefaultAlternative::new("x", 42.0)),
                    ),
                ] {
                    let context = inkwell::context::Context::create();
                    let type_compiler = TypeCompiler::new(&context);
                    let module = context.create_module("");
                    let function = module.add_function(
                        "",
                        context.f64_type().fn_type(
                            &[type_compiler.compile_value(&ssf::types::Primitive::Float64.into())],
                            false,
                        ),
                        None,
                    );
                    let builder = context.create_builder();
                    builder.position_at_end(context.append_basic_block(function, "entry"));

                    builder.build_return(Some(
                        &ExpressionCompiler::new(
                            &context,
                            &module,
                            &builder,
                            &FunctionCompiler::new(
                                &context,
                                &module,
                                &type_compiler,
                                &HashMap::new(),
                                &compile_configuration,
                            ),
                            &type_compiler,
                            &compile_configuration,
                        )
                        .compile(
                            &primitive_case.into(),
                            &vec![("x".into(), function.get_params()[0])]
                                .drain(..)
                                .collect(),
                        )
                        .unwrap(),
                    ));

                    assert!(function.verify(true));
                    assert!(module.verify().is_ok());
                }
            }
        }
    }

    mod constructor_applications {
        use super::*;

        #[test]
        fn compile_algebraic_case_expression_with_multiple_constructors() {
            let compile_configuration = CompileConfiguration::new("", vec![], None, None);
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
                    ssf::ir::Constructor::new(algebraic_type.clone(), 1),
                    vec![42.0.into()],
                ),
            ] {
                let context = inkwell::context::Context::create();
                let type_compiler = TypeCompiler::new(&context);
                let module = context.create_module("");

                module.add_function(
                    compile_configuration.malloc_function_name(),
                    context
                        .i8_type()
                        .ptr_type(inkwell::AddressSpace::Generic)
                        .fn_type(&[context.i64_type().into()], false),
                    None,
                );

                let function = module.add_function(
                    "",
                    type_compiler
                        .compile_algebraic(&algebraic_type, None)
                        .fn_type(&[], false),
                    None,
                );
                let builder = context.create_builder();
                builder.position_at_end(context.append_basic_block(function, "entry"));

                builder.build_return(Some(
                    &ExpressionCompiler::new(
                        &context,
                        &module,
                        &builder,
                        &FunctionCompiler::new(
                            &context,
                            &module,
                            &type_compiler,
                            &HashMap::new(),
                            &compile_configuration,
                        ),
                        &type_compiler,
                        &compile_configuration,
                    )
                    .compile(&constructor_application.into(), &HashMap::new())
                    .unwrap(),
                ));

                assert!(function.verify(true));
                assert!(module.verify().is_ok());
            }
        }
    }
}
