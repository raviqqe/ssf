use super::error::CompileError;
use super::function_compiler::FunctionCompiler;
use super::type_compiler::TypeCompiler;
use std::collections::HashMap;

pub struct ExpressionCompiler<'c, 'm, 'b, 'f, 't, 'v> {
    context: &'c inkwell::context::Context,
    module: &'m inkwell::module::Module<'c>,
    builder: &'b inkwell::builder::Builder<'c>,
    function_compiler: &'f FunctionCompiler<'c, 'm, 't, 'v>,
    type_compiler: &'t TypeCompiler<'c, 'm>,
}

impl<'c, 'm, 'b, 'f, 't, 'v> ExpressionCompiler<'c, 'm, 'b, 'f, 't, 'v> {
    pub fn new(
        context: &'c inkwell::context::Context,
        module: &'m inkwell::module::Module<'c>,
        builder: &'b inkwell::builder::Builder<'c>,
        function_compiler: &'f FunctionCompiler<'c, 'm, 't, 'v>,
        type_compiler: &'t TypeCompiler<'c, 'm>,
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
        expression: &ssf::ast::Expression,
        variables: &HashMap<String, inkwell::values::BasicValueEnum<'c>>,
    ) -> Result<inkwell::values::BasicValueEnum<'c>, CompileError> {
        match expression {
            ssf::ast::Expression::Application(application) => {
                let closure = self
                    .compile_variable(application.function(), variables)?
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

                for argument in application.arguments() {
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
            ssf::ast::Expression::LetFunctions(let_functions) => {
                let mut variables = variables.clone();
                let mut closures = HashMap::<&str, inkwell::values::PointerValue>::new();

                for definition in let_functions.definitions() {
                    let closure_type = self.type_compiler.compile_closure(definition);
                    let pointer = self
                        .builder
                        .build_call(
                            self.module.get_function("malloc").unwrap(),
                            &[closure_type.size_of().unwrap().into()],
                            "",
                        )
                        .try_as_basic_value()
                        .left()
                        .unwrap();

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
                    closures.insert(
                        definition.name(),
                        self.builder
                            .build_bitcast(
                                pointer,
                                closure_type.ptr_type(inkwell::AddressSpace::Generic),
                                "",
                            )
                            .into_pointer_value(),
                    );
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
            ssf::ast::Expression::LetValues(let_values) => {
                let mut variables = variables.clone();

                for definition in let_values.definitions() {
                    variables.insert(
                        definition.name().into(),
                        self.compile(definition.body(), &variables)?,
                    );
                }

                self.compile(let_values.expression(), &variables)
            }
            ssf::ast::Expression::Number(number) => {
                Ok(self.context.f64_type().const_float(*number).into())
            }
            ssf::ast::Expression::Operation(operation) => {
                let lhs = self.compile(operation.lhs(), variables)?.into_float_value();
                let rhs = self.compile(operation.rhs(), variables)?.into_float_value();

                Ok(match operation.operator() {
                    ssf::ast::Operator::Add => self.builder.build_float_add(lhs, rhs, ""),
                    ssf::ast::Operator::Subtract => self.builder.build_float_sub(lhs, rhs, ""),
                    ssf::ast::Operator::Multiply => self.builder.build_float_mul(lhs, rhs, ""),
                    ssf::ast::Operator::Divide => self.builder.build_float_div(lhs, rhs, ""),
                }
                .into())
            }
            ssf::ast::Expression::Variable(variable) => self.compile_variable(variable, variables),
        }
    }

    fn compile_variable(
        &self,
        variable: &ssf::ast::Variable,
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
}
