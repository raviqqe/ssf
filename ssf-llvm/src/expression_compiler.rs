use super::error::CompileError;
use super::function_compiler::FunctionCompiler;
use super::type_compiler::TypeCompiler;
use std::collections::HashMap;

pub struct ExpressionCompiler<'a, 'b> {
    context: &'b llvm::Context,
    builder: &'b llvm::Builder,
    function_compiler: &'b mut FunctionCompiler<'a>,
    type_compiler: &'a TypeCompiler<'a>,
}

impl<'a, 'b> ExpressionCompiler<'a, 'b> {
    pub fn new(
        context: &'b llvm::Context,
        builder: &'b llvm::Builder,
        function_compiler: &'b mut FunctionCompiler<'a>,
        type_compiler: &'a TypeCompiler<'a>,
    ) -> Self {
        Self {
            context,
            builder,
            function_compiler,
            type_compiler,
        }
    }

    pub fn compile(
        &mut self,
        expression: &ssf::ast::Expression,
        variables: &HashMap<String, llvm::Value>,
    ) -> Result<llvm::Value, CompileError> {
        match expression {
            ssf::ast::Expression::Application(application) => {
                let closure = self.compile_variable(application.function(), variables)?;

                let mut arguments = vec![self.builder.build_gep(
                    closure,
                    &[
                        llvm::const_int(self.context.i32_type(), 0),
                        llvm::const_int(self.context.i32_type(), 1),
                    ],
                )];

                for argument in application.arguments() {
                    arguments.push(self.compile(argument, variables)?);
                }

                Ok(self.builder.build_call(
                    self.builder.build_load(self.builder.build_gep(
                        closure,
                        &[
                            llvm::const_int(self.context.i32_type(), 0),
                            llvm::const_int(self.context.i32_type(), 0),
                        ],
                    )),
                    &arguments,
                ))
            }
            ssf::ast::Expression::LetFunctions(let_functions) => {
                let mut variables = variables.clone();
                let mut closures = HashMap::<&str, llvm::Value>::new();

                for definition in let_functions.definitions() {
                    let closure_type = self.type_compiler.compile_closure(definition);
                    let pointer = self.builder.build_malloc(closure_type.size());

                    variables.insert(
                        definition.name().into(),
                        self.builder.build_bit_cast(
                            pointer,
                            self.context.pointer_type(
                                self.type_compiler
                                    .compile_unsized_closure(definition.type_()),
                            ),
                        ),
                    );
                    closures.insert(
                        definition.name(),
                        self.builder
                            .build_bit_cast(pointer, self.context.pointer_type(closure_type)),
                    );
                }

                for definition in let_functions.definitions() {
                    let closure = closures[definition.name()];

                    self.builder.build_store(
                        self.function_compiler.compile(definition)?,
                        self.builder.build_gep(
                            closure,
                            &[
                                llvm::const_int(self.context.i32_type(), 0),
                                llvm::const_int(self.context.i32_type(), 0),
                            ],
                        ),
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
                            *value,
                            self.builder.build_gep(
                                closure,
                                &[
                                    llvm::const_int(self.context.i32_type(), 0),
                                    llvm::const_int(self.context.i32_type(), 1),
                                    llvm::const_int(self.context.i32_type(), index as u64),
                                ],
                            ),
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
                Ok(llvm::const_real(self.context.double_type(), *number))
            }
            ssf::ast::Expression::Operation(operation) => {
                let lhs = self.compile(operation.lhs(), variables)?;
                let rhs = self.compile(operation.rhs(), variables)?;

                Ok(match operation.operator() {
                    ssf::ast::Operator::Add => self.builder.build_fadd(lhs, rhs),
                    ssf::ast::Operator::Subtract => self.builder.build_fsub(lhs, rhs),
                    ssf::ast::Operator::Multiply => self.builder.build_fmul(lhs, rhs),
                    ssf::ast::Operator::Divide => self.builder.build_fdiv(lhs, rhs),
                })
            }
            ssf::ast::Expression::Variable(variable) => self.compile_variable(variable, variables),
        }
    }

    fn compile_variable(
        &self,
        variable: &ssf::ast::Variable,
        variables: &HashMap<String, llvm::Value>,
    ) -> Result<llvm::Value, CompileError> {
        match variables.get(variable.name()) {
            Some(value) => Ok(self.unwrap_value(*value)),
            None => Err(CompileError::VariableNotFound),
        }
    }

    fn unwrap_value(&self, value: llvm::Value) -> llvm::Value {
        if value.type_().kind() == llvm::TypeKind::Pointer {
            match value.type_().element().kind() {
                llvm::TypeKind::Double => self.builder.build_load(value),
                _ => value,
            }
        } else {
            value
        }
    }
}
