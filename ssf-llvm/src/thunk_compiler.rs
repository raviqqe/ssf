use super::type_compiler::TypeCompiler;
use inkwell::types::BasicType;
use inkwell::values::BasicValue;

pub struct ThunkCompiler<'c, 'm, 't> {
    context: &'c inkwell::context::Context,
    module: &'m inkwell::module::Module<'c>,
    type_compiler: &'t TypeCompiler<'c>,
}

impl<'c, 'm, 't> ThunkCompiler<'c, 'm, 't> {
    pub fn new(
        context: &'c inkwell::context::Context,
        module: &'m inkwell::module::Module<'c>,
        type_compiler: &'t TypeCompiler<'c>,
    ) -> Self {
        Self {
            context,
            module,
            type_compiler,
        }
    }

    pub fn compile_normal_entry(
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

        self.compile_body(&builder, entry_function);

        entry_function.verify(true);

        entry_function
    }

    pub fn compile_locked_entry(
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

        let current_entry_function = builder.build_load(
            unsafe {
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
            },
            "",
        );
        current_entry_function
            .as_instruction_value()
            .unwrap()
            .set_alignment(8)
            .unwrap();
        current_entry_function
            .as_instruction_value()
            .unwrap()
            .set_atomic_ordering(inkwell::AtomicOrdering::SequentiallyConsistent)
            .unwrap();

        let condition = builder.build_int_compare(
            inkwell::IntPredicate::EQ,
            builder.build_ptr_to_int(
                current_entry_function.into_pointer_value(),
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

        builder.build_conditional_branch(condition, loop_block, final_block);

        builder.position_at_end(final_block);

        self.compile_body(&builder, entry_function);

        entry_function.verify(true);

        entry_function
    }

    fn compile_body(
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

    fn generate_normal_entry_name(name: &str) -> String {
        [name, ".$entry.normal"].concat()
    }

    fn generate_locked_entry_name(name: &str) -> String {
        [name, ".$entry.locked"].concat()
    }
}
