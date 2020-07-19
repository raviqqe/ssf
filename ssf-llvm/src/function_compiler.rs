use super::compile_configuration::CompileConfiguration;
use super::error::CompileError;
use super::expression_compiler::ExpressionCompiler;
use super::type_compiler::TypeCompiler;
use inkwell::types::BasicType;
use inkwell::values::BasicValue;
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
        Ok(if function_definition.arguments().is_empty() {
            self.compile_thunk(function_definition)?
        } else {
            self.compile_non_thunk(function_definition)?
        })
    }

    fn compile_non_thunk(
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

        let entry_pointer = self.compile_entry_pointer(&builder, entry_function);

        let condition = builder
            .build_cmpxchg(
                entry_pointer,
                entry_function.as_global_value().as_pointer_value(),
                self.compile_locked_entry(function_definition)
                    .as_global_value()
                    .as_pointer_value(),
                inkwell::AtomicOrdering::SequentiallyConsistent,
                inkwell::AtomicOrdering::SequentiallyConsistent,
            )
            .unwrap();

        let then_block = self.context.append_basic_block(entry_function, "then");
        let else_block = self.context.append_basic_block(entry_function, "else");

        builder.build_conditional_branch(
            builder
                .build_extract_value(condition, 1, "")
                .unwrap()
                .into_int_value(),
            then_block,
            else_block,
        );

        builder.position_at_end(else_block);

        builder.build_return(Some(
            &builder
                .build_call(
                    self.compile_atomic_load(&builder, entry_pointer),
                    &[entry_function.get_params()[0]],
                    "",
                )
                .try_as_basic_value()
                .left()
                .unwrap(),
        ));

        builder.position_at_end(then_block);

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
            self.compile_normal_entry(function_definition)
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

        Ok(ExpressionCompiler::new(
            self.context,
            self.module,
            &builder,
            self,
            self.type_compiler,
            self.compile_configuration,
        )
        .compile(&function_definition.body(), &variables)?)
    }

    fn compile_normal_entry(
        &self,
        function_definition: &ssf::ir::FunctionDefinition,
    ) -> inkwell::values::FunctionValue {
        let entry_function = self.module.add_function(
            &Self::generate_normal_entry_name(function_definition.name()),
            self.type_compiler
                .compile_entry_function(function_definition.type_()),
            None,
        );

        let builder = self.context.create_builder();
        builder.position_at_end(self.context.append_basic_block(entry_function, "entry"));

        self.compile_normal_body(&builder, entry_function);

        entry_function.verify(true);

        entry_function
    }

    fn compile_locked_entry(
        &self,
        function_definition: &ssf::ir::FunctionDefinition,
    ) -> inkwell::values::FunctionValue<'c> {
        let entry_function = self.module.add_function(
            &Self::generate_locked_entry_name(function_definition.name()),
            self.type_compiler
                .compile_entry_function(function_definition.type_()),
            None,
        );

        let builder = self.context.create_builder();

        let entry_block = self.context.append_basic_block(entry_function, "entry");
        let loop_block = self.context.append_basic_block(entry_function, "loop");

        builder.position_at_end(entry_block);
        builder.build_unconditional_branch(loop_block);
        builder.position_at_end(loop_block);

        let current_entry_function = self.compile_atomic_load(
            &builder,
            self.compile_entry_pointer(&builder, entry_function),
        );

        let condition = builder.build_int_compare(
            inkwell::IntPredicate::EQ,
            builder.build_ptr_to_int(current_entry_function, self.context.i64_type(), ""),
            builder.build_ptr_to_int(
                entry_function.as_global_value().as_pointer_value(),
                self.context.i64_type(),
                "",
            ),
            "",
        );

        let final_block = self.context.append_basic_block(entry_function, "final");

        builder.build_conditional_branch(condition, loop_block, final_block);

        builder.position_at_end(final_block);

        self.compile_normal_body(&builder, entry_function);

        entry_function.verify(true);

        entry_function
    }

    fn compile_normal_body(
        &self,
        builder: &inkwell::builder::Builder<'c>,
        entry_function: inkwell::values::FunctionValue<'c>,
    ) {
        builder.build_return(Some(
            &builder.build_load(
                builder
                    .build_bitcast(
                        entry_function.get_params()[0],
                        entry_function
                            .get_type()
                            .get_return_type()
                            .unwrap()
                            .ptr_type(inkwell::AddressSpace::Generic),
                        "",
                    )
                    .into_pointer_value(),
                "",
            ),
        ));
    }

    fn compile_entry_pointer(
        &self,
        builder: &inkwell::builder::Builder<'c>,
        entry_function: inkwell::values::FunctionValue<'c>,
    ) -> inkwell::values::PointerValue<'c> {
        let base_pointer = builder
            .build_bitcast(
                entry_function.get_params()[0],
                entry_function
                    .get_type()
                    .ptr_type(inkwell::AddressSpace::Generic)
                    .ptr_type(inkwell::AddressSpace::Generic),
                "",
            )
            .into_pointer_value();

        unsafe {
            builder.build_gep(
                base_pointer,
                &[self.context.i64_type().const_int(-1i64 as u64, true)],
                "",
            )
        }
    }

    fn compile_atomic_load(
        &self,
        builder: &inkwell::builder::Builder<'c>,
        pointer: inkwell::values::PointerValue<'c>,
    ) -> inkwell::values::PointerValue<'c> {
        let value = builder.build_load(pointer, "");

        value
            .as_instruction_value()
            .unwrap()
            .set_alignment(8)
            .unwrap();
        value
            .as_instruction_value()
            .unwrap()
            .set_atomic_ordering(inkwell::AtomicOrdering::SequentiallyConsistent)
            .unwrap();

        value.into_pointer_value()
    }

    fn generate_closure_entry_name(name: &str) -> String {
        [name, ".$entry"].concat()
    }

    fn generate_normal_entry_name(name: &str) -> String {
        [name, ".$entry.normal"].concat()
    }

    fn generate_locked_entry_name(name: &str) -> String {
        [name, ".$entry.locked"].concat()
    }
}
