use super::error::CompileError;
use super::expression_compiler_factory::ExpressionCompilerFactory;
use super::global_variable::GlobalVariable;
use super::instruction_compiler::InstructionCompiler;
use super::type_compiler::TypeCompiler;
use super::utilities;
use inkwell::types::BasicType;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct FunctionCompiler<'c> {
    context: &'c inkwell::context::Context,
    module: Arc<inkwell::module::Module<'c>>,
    expression_compiler_factory: Arc<ExpressionCompilerFactory<'c>>,
    type_compiler: Arc<TypeCompiler<'c>>,
    global_variables: Arc<HashMap<String, GlobalVariable<'c>>>,
}

impl<'c> FunctionCompiler<'c> {
    pub fn new(
        context: &'c inkwell::context::Context,
        module: Arc<inkwell::module::Module<'c>>,
        expression_compiler_factory: Arc<ExpressionCompilerFactory<'c>>,
        type_compiler: Arc<TypeCompiler<'c>>,
        global_variables: Arc<HashMap<String, GlobalVariable<'c>>>,
    ) -> Arc<Self> {
        Self {
            context,
            module,
            expression_compiler_factory,
            type_compiler,
            global_variables,
        }
        .into()
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

    fn compile_non_thunk(
        &self,
        definition: &ssf::ir::Definition,
    ) -> Result<inkwell::values::FunctionValue<'c>, CompileError> {
        let entry_function = self.add_function(
            &Self::generate_closure_entry_name(definition.name()),
            self.type_compiler
                .compile_entry_function_from_definition(definition),
        );

        let builder = Arc::new(self.context.create_builder());
        builder.position_at_end(self.context.append_basic_block(entry_function, "entry"));

        builder.build_return(Some(&self.compile_body(builder.clone(), definition)?));

        entry_function.verify(true);

        Ok(entry_function)
    }

    fn compile_thunk(
        &self,
        definition: &ssf::ir::Definition,
    ) -> Result<inkwell::values::FunctionValue<'c>, CompileError> {
        let entry_function = self.add_function(
            &Self::generate_closure_entry_name(definition.name()),
            self.type_compiler
                .compile_entry_function_from_definition(definition),
        );

        let builder = Arc::new(self.context.create_builder());
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

        let lock_success_block = self.context.append_basic_block(entry_function, "then");
        let lock_failure_block = self.context.append_basic_block(entry_function, "else");

        builder.build_conditional_branch(
            builder
                .build_extract_value(condition, 1, "")
                .unwrap()
                .into_int_value(),
            lock_success_block,
            lock_failure_block,
        );

        builder.position_at_end(lock_failure_block);

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

        builder.position_at_end(lock_success_block);

        let result = self.compile_body(builder.clone(), definition)?;

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
        builder: Arc<inkwell::builder::Builder<'c>>,
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
            .map(|(name, global_variable)| (name.into(), global_variable.load(&builder).into()))
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

        Ok(self
            .expression_compiler_factory
            .create(builder, self.clone().into())
            .compile(&definition.body(), &variables)?)
    }

    fn compile_normal_entry(
        &self,
        definition: &ssf::ir::Definition,
    ) -> inkwell::values::FunctionValue<'c> {
        let entry_function = self.add_function(
            &Self::generate_normal_entry_name(definition.name()),
            self.type_compiler
                .compile_entry_function_from_definition(definition),
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
        let entry_function = self.add_function(
            &Self::generate_locked_entry_name(definition.name()),
            self.type_compiler
                .compile_entry_function_from_definition(definition),
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
                &[self.context.i64_type().const_int(-2i64 as u64, true)],
                "",
            )
        }
    }

    fn add_function(
        &self,
        name: &str,
        type_: inkwell::types::FunctionType<'c>,
    ) -> inkwell::values::FunctionValue<'c> {
        utilities::add_function_to_module(self.module.clone(), name, type_)
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
