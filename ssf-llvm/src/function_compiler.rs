use super::compile_configuration::CompileConfiguration;
use super::error::CompileError;
use super::expression_compiler::ExpressionCompiler;
use super::thunk_compiler::ThunkCompiler;
use super::type_compiler::TypeCompiler;
use inkwell::types::BasicType;
use std::collections::HashMap;

pub struct FunctionCompiler<'c, 'm, 't, 'v> {
    thunk_compiler: ThunkCompiler<'c, 'm, 't>,
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
            thunk_compiler: ThunkCompiler::new(context, module, type_compiler),
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
        Ok(if function_definition.arguments().is_empty() {
            self.compile_thunk(function_definition)?
        } else {
            self.compile_function(function_definition)?
        })
    }

    fn compile_function(
        &self,
        function_definition: &ssf::ir::FunctionDefinition,
    ) -> Result<inkwell::values::FunctionValue, CompileError> {
        let entry_function = self.module.add_function(
            &Self::generate_closure_entry_name(function_definition.name()),
            self.type_compiler
                .compile_entry_function(function_definition.type_()),
            None,
        );

        let builder = self.context.create_builder();
        builder.position_at_end(self.context.append_basic_block(entry_function, "entry"));

        builder.build_return(Some(&self.compile_body(&builder, function_definition)?));

        entry_function.verify(true);

        Ok(entry_function)
    }

    fn compile_thunk(
        &self,
        function_definition: &ssf::ir::FunctionDefinition,
    ) -> Result<inkwell::values::FunctionValue, CompileError> {
        let entry_function = self.module.add_function(
            &Self::generate_closure_entry_name(function_definition.name()),
            self.type_compiler
                .compile_entry_function(function_definition.type_()),
            None,
        );

        let builder = self.context.create_builder();
        builder.position_at_end(self.context.append_basic_block(entry_function, "entry"));

        let entry_pointer = unsafe {
            builder.build_gep(
                builder
                    .build_bitcast(
                        entry_function.get_params()[0],
                        entry_function
                            .get_type()
                            .ptr_type(inkwell::AddressSpace::Generic)
                            .ptr_type(inkwell::AddressSpace::Generic),
                        "",
                    )
                    .into_pointer_value(),
                &[self.context.i64_type().const_int(-1i64 as u64, true)],
                "",
            )
        };

        let condition = builder
            .build_cmpxchg(
                entry_pointer,
                entry_function.as_global_value().as_pointer_value(),
                self.thunk_compiler
                    .compile_locked_entry(function_definition)
                    .as_global_value()
                    .as_pointer_value(),
                inkwell::AtomicOrdering::SequentiallyConsistent,
                inkwell::AtomicOrdering::SequentiallyConsistent,
            )
            .unwrap();

        let next_block = self.context.append_basic_block(entry_function, "next");

        builder.build_conditional_branch(
            builder
                .build_extract_value(condition, 1, "")
                .unwrap()
                .into_int_value(),
            next_block,
            entry_block,
        );

        builder.position_at_end(next_block);

        let result = self.compile_body(&builder, function_definition)?;

        builder.build_store(
            builder
                .build_bitcast(
                    entry_function.get_params()[0],
                    result.get_type().ptr_type(inkwell::AddressSpace::Generic),
                    "",
                )
                .into_pointer_value(),
            result,
        );

        let store_value = builder.build_store(
            entry_pointer,
            self.thunk_compiler
                .compile_normal_entry(function_definition)
                .as_global_value()
                .as_pointer_value(),
        );
        store_value.set_alignment(8).unwrap();
        store_value
            .set_atomic_ordering(inkwell::AtomicOrdering::SequentiallyConsistent)
            .unwrap();

        builder.build_return(Some(&result));

        entry_function.verify(true);

        Ok(entry_function)
    }

    fn compile_body(
        &self,
        builder: &inkwell::builder::Builder<'c>,
        function_definition: &ssf::ir::FunctionDefinition,
    ) -> Result<inkwell::values::BasicValueEnum<'c>, CompileError> {
        let entry_function = builder.get_insert_block().unwrap().get_parent().unwrap();

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

        Ok(expression_compiler.compile(&function_definition.body(), &variables)?)
    }

    fn generate_closure_entry_name(name: &str) -> String {
        [name, ".$entry"].concat()
    }
}
