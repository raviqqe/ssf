use super::error::CompileError;
use super::function_compiler::FunctionCompiler;
use super::type_compiler::TypeCompiler;
use inkwell::values::BasicValue;
use std::collections::HashMap;

pub struct ExpressionCompiler<'c, 'm, 'b, 'f, 't, 'v> {
    context: &'c inkwell::context::Context,
    module: &'m inkwell::module::Module<'c>,
    builder: &'b inkwell::builder::Builder<'c>,
    function_compiler: &'f FunctionCompiler<'c, 'm, 't, 'v>,
    type_compiler: &'t TypeCompiler<'c>,
}

impl<'c, 'm, 'b, 'f, 't, 'v> ExpressionCompiler<'c, 'm, 'b, 'f, 't, 'v> {
    pub fn new(
        context: &'c inkwell::context::Context,
        module: &'m inkwell::module::Module<'c>,
        builder: &'b inkwell::builder::Builder<'c>,
        function_compiler: &'f FunctionCompiler<'c, 'm, 't, 'v>,
        type_compiler: &'t TypeCompiler<'c>,
    ) -> Self {
        Self {
            context,
            module,
            builder,
            function_compiler,
            type_compiler,
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
                let constructor_type = self
                    .type_compiler
                    .compile_constructor(constructor_application.constructor().constructor_type());

                let constructor_pointer = if constructor_application
                    .constructor()
                    .constructor_type()
                    .elements()
                    .len()
                    == 0
                {
                    self.type_compiler
                        .compile_unsized_constructor()
                        .const_null()
                } else {
                    let constructor_pointer = self.compile_struct_malloc(
                        constructor_type.get_element_type().into_struct_type(),
                    );

                    let mut constructor_value = constructor_type
                        .get_element_type()
                        .into_struct_type()
                        .const_zero()
                        .into();

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

                    self.builder
                        .build_store(constructor_pointer, constructor_value);

                    constructor_pointer
                };

                let mut algebraic_value = self
                    .type_compiler
                    .compile_algebraic(constructor_application.constructor().algebraic_type())
                    .const_zero()
                    .into();

                algebraic_value = self
                    .builder
                    .build_insert_value(
                        algebraic_value,
                        self.context
                            .i64_type()
                            .const_int(constructor_application.constructor().index() as u64, false),
                        0,
                        "",
                    )
                    .unwrap();

                Ok(self
                    .builder
                    .build_insert_value(
                        algebraic_value,
                        self.builder.build_bitcast(
                            constructor_pointer,
                            algebraic_value
                                .into_struct_value()
                                .get_type()
                                .get_field_type_at_index(1)
                                .unwrap(),
                            "",
                        ),
                        1,
                        "",
                    )
                    .unwrap()
                    .as_basic_value_enum())
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
            ssf::ir::Expression::Float64(number) => {
                Ok(self.context.f64_type().const_float(*number).into())
            }
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
                let tag = if algebraic_case.alternatives().is_empty()
                    || algebraic_case.alternatives()[0]
                        .constructor()
                        .algebraic_type()
                        .constructors()
                        .len()
                        == 1
                {
                    self.context.i64_type().const_int(0, false).into()
                } else {
                    self.builder.build_extract_value(argument, 0, "").unwrap()
                }
                .into_int_value();

                let switch_block = self.builder.get_insert_block().unwrap();
                let phi_block = self.append_basic_block("phi");
                let mut cases = vec![];

                for (index, alternative) in algebraic_case.alternatives().iter().enumerate() {
                    let block = self.append_basic_block(&format!("case.{}", index));
                    self.builder.position_at_end(&block);

                    let elements = self
                        .builder
                        .build_load(
                            self.builder
                                .build_bitcast(
                                    if alternative.constructor().algebraic_type().is_singleton() {
                                        self.builder.build_extract_value(argument, 0, "").unwrap()
                                    } else {
                                        self.builder.build_extract_value(argument, 1, "").unwrap()
                                    },
                                    self.type_compiler.compile_constructor(
                                        alternative.constructor().constructor_type(),
                                    ),
                                    "",
                                )
                                .into_pointer_value(),
                            "",
                        )
                        .into_struct_value();

                    let mut variables = variables.clone();

                    for (index, name) in alternative.element_names().iter().enumerate() {
                        variables.insert(
                            name.into(),
                            self.builder
                                .build_extract_value(elements, index as u32, "")
                                .unwrap(),
                        );
                    }

                    cases.push((
                        self.context
                            .i64_type()
                            .const_int(alternative.constructor().index() as u64, false),
                        block,
                        self.compile(alternative.expression(), &variables)?,
                    ));

                    self.builder.build_unconditional_branch(&phi_block);
                }

                let default_block = self.append_basic_block("default");
                self.builder.position_at_end(&default_block);
                let default_value = match algebraic_case.default_alternative() {
                    None => {
                        self.builder.build_unreachable();
                        None
                    }
                    Some(default_alternative) => {
                        let mut variables = variables.clone();

                        variables.insert(default_alternative.variable().into(), argument.into());

                        let value = self.compile(default_alternative.expression(), &variables)?;
                        self.builder.build_unconditional_branch(&phi_block);
                        Some(value)
                    }
                };

                self.builder.position_at_end(&switch_block);
                self.builder.build_switch(
                    tag,
                    &default_block,
                    &cases
                        .iter()
                        .map(|(tag, block, _)| (*tag, block))
                        .collect::<Vec<_>>(),
                );

                self.builder.position_at_end(&phi_block);
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
                            (value as &dyn inkwell::values::BasicValue<'c>, block)
                        })
                        .chain(match &default_value {
                            Some(default_value) => vec![(
                                default_value as &dyn inkwell::values::BasicValue<'c>,
                                &default_block,
                            )],
                            None => vec![],
                        })
                        .collect::<Vec<_>>(),
                );

                Ok(phi.as_basic_value())
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

    fn append_basic_block(&self, name: &str) -> inkwell::basic_block::BasicBlock {
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
                        self.module.get_function("malloc").unwrap(),
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
}

#[cfg(test)]
mod tests {
    use super::*;

    mod case_expressions {
        use super::*;

        #[test]
        fn compile_algebraic_case_expression_with_multiple_constructors() {
            let algebraic_type = ssf::types::Algebraic::new(vec![
                ssf::types::Constructor::new(vec![]),
                ssf::types::Constructor::new(vec![ssf::types::Value::Float64.into()]),
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
                builder.position_at_end(&context.append_basic_block(function, "entry"));

                ExpressionCompiler::new(
                    &context,
                    &module,
                    &builder,
                    &FunctionCompiler::new(&context, &module, &type_compiler, &HashMap::new()),
                    &type_compiler,
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
            let algebraic_type =
                ssf::types::Algebraic::new(vec![ssf::types::Constructor::new(vec![])]);

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
                builder.position_at_end(&context.append_basic_block(function, "entry"));

                ExpressionCompiler::new(
                    &context,
                    &module,
                    &builder,
                    &FunctionCompiler::new(&context, &module, &type_compiler, &HashMap::new()),
                    &type_compiler,
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

    mod constructor_applications {
        use super::*;

        #[test]
        fn compile_algebraic_case_expression_with_multiple_constructors() {
            let algebraic_type = ssf::types::Algebraic::new(vec![
                ssf::types::Constructor::new(vec![]),
                ssf::types::Constructor::new(vec![ssf::types::Value::Float64.into()]),
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
                    "malloc",
                    context
                        .i8_type()
                        .ptr_type(inkwell::AddressSpace::Generic)
                        .fn_type(&[context.i64_type().into()], false),
                    None,
                );

                let function = module.add_function(
                    "",
                    type_compiler
                        .compile_algebraic(&algebraic_type)
                        .fn_type(&[], false),
                    None,
                );
                let builder = context.create_builder();
                builder.position_at_end(&context.append_basic_block(function, "entry"));

                builder.build_return(Some(
                    &ExpressionCompiler::new(
                        &context,
                        &module,
                        &builder,
                        &FunctionCompiler::new(&context, &module, &type_compiler, &HashMap::new()),
                        &type_compiler,
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
