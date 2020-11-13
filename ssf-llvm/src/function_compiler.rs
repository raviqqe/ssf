use super::compile_configuration::CompileConfiguration;
use super::error::CompileError;
use super::expression_compiler::ExpressionCompiler;
use super::instruction_compiler::InstructionCompiler;
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
        definition: &ssf::ir::Definition,
    ) -> Result<inkwell::values::FunctionValue<'c>, CompileError> {
        Ok(if definition.is_thunk() {
            self.compile_thunk(definition)?
        } else {
            self.compile_non_thunk(definition)?
        })
    }

    pub fn compile_partial_application(
        &self,
        function_type: inkwell::types::FunctionType<'c>,
        environment_type: inkwell::types::StructType<'c>,
    ) -> Result<inkwell::values::FunctionValue<'c>, CompileError> {
        let entry_function = self
            .module
            .add_function("partial_application", function_type, None);

        let builder = self.context.create_builder();
        builder.position_at_end(self.context.append_basic_block(entry_function, "entry"));

        builder.build_return(Some(&self.compile_partial_application_body(
            &builder,
            entry_function,
            environment_type,
        )?));

        entry_function.verify(true);

        Ok(entry_function)
    }

    fn compile_partial_application_body(
        &self,
        builder: &inkwell::builder::Builder<'c>,
        entry_function: inkwell::values::FunctionValue<'c>,
        environment_type: inkwell::types::StructType<'c>,
    ) -> Result<inkwell::values::BasicValueEnum<'c>, CompileError> {
        let environment = builder
            .build_load(
                builder
                    .build_bitcast(
                        entry_function.get_params()[0],
                        environment_type.ptr_type(inkwell::AddressSpace::Generic),
                        "",
                    )
                    .into_pointer_value(),
                "",
            )
            .into_struct_value();

        builder.build_unreachable();
        // TODO Cache partial application functions.
        // Ok(ExpressionCompiler::new(
        //     self.context,
        //     self.module,
        //     &builder,
        //     self,
        //     self.type_compiler,
        //     self.compile_configuration,
        // )
        // .compile_closure_call(
        //     builder
        //         .build_extract_value(environment, 0, "")
        //         .unwrap()
        //         .into_pointer_value(),
        //     &(1..environment.get_type().count_fields())
        //         .map(|index| builder.build_extract_value(environment, index, "").unwrap())
        //         .collect::<Vec<_>>(),
        // )?)
    }

    fn compile_non_thunk(
        &self,
        definition: &ssf::ir::Definition,
    ) -> Result<inkwell::values::FunctionValue<'c>, CompileError> {
        let entry_function = self.module.add_function(
            &Self::generate_closure_entry_name(definition.name()),
            self.type_compiler.compile_entry_function(definition),
            None,
        );

        let builder = self.context.create_builder();
        builder.position_at_end(self.context.append_basic_block(entry_function, "entry"));

        builder.build_return(Some(&self.compile_body(&builder, definition)?));

        entry_function.verify(true);

        Ok(entry_function)
    }

    fn compile_thunk(
        &self,
        definition: &ssf::ir::Definition,
    ) -> Result<inkwell::values::FunctionValue<'c>, CompileError> {
        let entry_function = self.module.add_function(
            &Self::generate_closure_entry_name(definition.name()),
            self.type_compiler.compile_entry_function(definition),
            None,
        );

        let builder = self.context.create_builder();
        builder.position_at_end(self.context.append_basic_block(entry_function, "entry"));

        let entry_pointer = self.compile_entry_pointer(&builder, entry_function);

        let condition = builder
            .build_cmpxchg(
                entry_pointer,
                entry_function.as_global_value().as_pointer_value(),
                self.compile_locked_entry(definition)
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
                    builder
                        .build_extract_value(condition, 0, "")
                        .unwrap()
                        .into_pointer_value(),
                    &entry_function.get_params(),
                    "",
                )
                .try_as_basic_value()
                .left()
                .unwrap(),
        ));

        builder.position_at_end(then_block);

        let result = self.compile_body(&builder, definition)?;

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

        InstructionCompiler::compile_atomic_store(
            &builder,
            entry_pointer,
            self.compile_normal_entry(definition)
                .as_global_value()
                .as_pointer_value(),
        );

        builder.build_return(Some(&result));

        entry_function.verify(true);

        Ok(entry_function)
    }

    fn compile_body(
        &self,
        builder: &inkwell::builder::Builder<'c>,
        definition: &ssf::ir::Definition,
    ) -> Result<inkwell::values::BasicValueEnum<'c>, CompileError> {
        let entry_function = builder.get_insert_block().unwrap().get_parent().unwrap();

        let environment = builder
            .build_bitcast(
                entry_function.get_params()[0],
                self.type_compiler
                    .compile_environment(definition)
                    .ptr_type(inkwell::AddressSpace::Generic),
                "",
            )
            .into_pointer_value();

        let mut variables = self
            .global_variables
            .iter()
            .map(|(name, value)| (name.into(), value.as_pointer_value().into()))
            .collect::<HashMap<String, inkwell::values::BasicValueEnum>>();

        for (index, free_variable) in definition.environment().iter().enumerate() {
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

        for (index, argument) in definition.arguments().iter().enumerate() {
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
        .compile(&definition.body(), &variables)?)
    }

    fn compile_normal_entry(
        &self,
        definition: &ssf::ir::Definition,
    ) -> inkwell::values::FunctionValue<'c> {
        let entry_function = self.module.add_function(
            &Self::generate_normal_entry_name(definition.name()),
            self.type_compiler.compile_entry_function(definition),
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
        definition: &ssf::ir::Definition,
    ) -> inkwell::values::FunctionValue<'c> {
        let entry_function = self.module.add_function(
            &Self::generate_locked_entry_name(definition.name()),
            self.type_compiler.compile_entry_function(definition),
            None,
        );

        let builder = self.context.create_builder();

        let entry_block = self.context.append_basic_block(entry_function, "entry");
        let loop_block = self.context.append_basic_block(entry_function, "loop");

        builder.position_at_end(entry_block);
        builder.build_unconditional_branch(loop_block);
        builder.position_at_end(loop_block);

        let condition = builder.build_int_compare(
            inkwell::IntPredicate::EQ,
            builder.build_ptr_to_int(
                InstructionCompiler::compile_atomic_load(
                    &builder,
                    self.compile_entry_pointer(&builder, entry_function),
                )
                .into_pointer_value(),
                self.context.i64_type(),
                "",
            ),
            builder.build_ptr_to_int(
                entry_function.as_global_value().as_pointer_value(),
                self.context.i64_type(),
                "",
            ),
            "",
        );

        let final_block = self.context.append_basic_block(entry_function, "final");

        // TODO Do not spin-lock.
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
