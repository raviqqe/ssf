use super::compile_configuration::CompileConfiguration;
use super::error::CompileError;
use super::expression_compiler::ExpressionCompiler;
use super::type_compiler::TypeCompiler;
use inkwell::types::BasicType;
use std::collections::HashMap;

pub struct FunctionCompiler<'c, 'm, 't, 'v> {
    context: &'c inkwell::context::Context,
    module: &'m inkwell::module::Module<'c>,
    type_compiler: &'t TypeCompiler<'c>,
    global_variables: &'v HashMap<String, inkwell::values::GlobalValue<'c>>,
    compile_configuration: &'c CompileConfiguration,
}

impl<'c, 'm, 't, 'v> FunctionCompiler<'c, 'm, 't, 'v> {
    pub fn new(
        context: &'c inkwell::context::Context,
        module: &'m inkwell::module::Module<'c>,
        type_compiler: &'t TypeCompiler<'c>,
        global_variables: &'v HashMap<String, inkwell::values::GlobalValue<'c>>,
        compile_configuration: &'c CompileConfiguration,
    ) -> Self {
        Self {
            context,
            module,
            type_compiler,
            global_variables,
            compile_configuration,
        }
    }

    pub fn compile(
        &self,
        function_definition: &ssf::ir::FunctionDefinition,
    ) -> Result<inkwell::values::FunctionValue, CompileError> {
        let entry_function_type = self
            .type_compiler
            .compile_entry_function(function_definition.type_());

        let entry_function = self.module.add_function(
            &Self::generate_closure_entry_name(function_definition.name()),
            entry_function_type,
            None,
        );

        let builder = self.context.create_builder();
        builder.position_at_end(self.context.append_basic_block(entry_function, "entry"));

        let environment = builder
            .build_bitcast(
                entry_function.get_params()[0],
                self.type_compiler
                    .compile_environment(function_definition)
                    .ptr_type(inkwell::AddressSpace::Generic),
                "",
            )
            .into_pointer_value();

        let mut variables = self
            .global_variables
            .iter()
            .map(|(name, value)| (name.into(), value.as_pointer_value().into()))
            .collect::<HashMap<String, inkwell::values::BasicValueEnum>>();

        for (index, free_variable) in function_definition.environment().iter().enumerate() {
            variables.insert(
                free_variable.name().into(),
                builder.build_load(
                    unsafe {
                        builder.build_gep(
                            environment,
                            &[
                                self.context.i32_type().const_int(0, false),
                                self.context.i32_type().const_int(index as u64, false),
                            ],
                            "",
                        )
                    },
                    "",
                ),
            );
        }

        for (index, argument) in function_definition.arguments().iter().enumerate() {
            variables.insert(
                argument.name().into(),
                entry_function.get_params()[index + 1],
            );
        }

        let expression_compiler = ExpressionCompiler::new(
            self.context,
            self.module,
            &builder,
            self,
            self.type_compiler,
            self.compile_configuration,
        );

        let result = expression_compiler.compile(&function_definition.body(), &variables)?;

        if function_definition.arguments().is_empty() {
            // TODO Make this thunk update thread-safe.
            builder.build_store(
                builder
                    .build_bitcast(
                        environment,
                        result.get_type().ptr_type(inkwell::AddressSpace::Generic),
                        "",
                    )
                    .into_pointer_value(),
                result,
            );

            builder.build_store(
                unsafe {
                    builder.build_gep(
                        builder
                            .build_bitcast(
                                environment,
                                entry_function_type
                                    .ptr_type(inkwell::AddressSpace::Generic)
                                    .ptr_type(inkwell::AddressSpace::Generic),
                                "",
                            )
                            .into_pointer_value(),
                        &[self.context.i64_type().const_int(-1i64 as u64, true)],
                        "",
                    )
                },
                self.compile_normal_thunk_entry(function_definition)
                    .as_global_value()
                    .as_pointer_value(),
            );
        }

        builder.build_return(Some(&result));

        entry_function.verify(true);

        Ok(entry_function)
    }

    fn compile_normal_thunk_entry(
        &self,
        function_definition: &ssf::ir::FunctionDefinition,
    ) -> inkwell::values::FunctionValue {
        let entry_function_type = self
            .type_compiler
            .compile_entry_function(function_definition.type_());

        let entry_function = self.module.add_function(
            &Self::generate_normal_thunk_entry_name(function_definition.name()),
            entry_function_type,
            None,
        );

        let builder = self.context.create_builder();
        builder.position_at_end(self.context.append_basic_block(entry_function, "entry"));

        let environment = builder
            .build_bitcast(
                entry_function.get_params()[0],
                entry_function_type
                    .get_return_type()
                    .unwrap()
                    .ptr_type(inkwell::AddressSpace::Generic),
                "",
            )
            .into_pointer_value();

        builder.build_return(Some(&builder.build_load(environment, "")));

        entry_function.verify(true);

        entry_function
    }

    fn generate_closure_entry_name(name: &str) -> String {
        [name, ".$entry"].concat()
    }

    fn generate_normal_thunk_entry_name(name: &str) -> String {
        [name, ".$entry.normal"].concat()
    }
}
