use super::error::CompileError;
use super::instruction_compiler::InstructionCompiler;
use super::type_compiler::TypeCompiler;
use super::utilities;
use std::sync::Arc;

pub struct ClosureOperationCompiler<'c> {
    context: &'c inkwell::context::Context,
    type_compiler: Arc<TypeCompiler<'c>>,
}

impl<'c> ClosureOperationCompiler<'c> {
    pub fn new(
        context: &'c inkwell::context::Context,
        type_compiler: Arc<TypeCompiler<'c>>,
    ) -> Arc<Self> {
        Self {
            context,
            type_compiler,
        }
        .into()
    }

    pub fn compile_load_entry_pointer(
        &self,
        builder: &inkwell::builder::Builder<'c>,
        closure: inkwell::values::PointerValue<'c>,
    ) -> inkwell::values::PointerValue<'c> {
        // Entry functions of thunks need to be loaded atomically
        // to make thunk update thread-safe.
        InstructionCompiler::compile_atomic_load(&builder, unsafe {
            builder.build_gep(
                closure,
                &[
                    self.context.i32_type().const_int(0, false),
                    self.context.i32_type().const_int(0, false),
                ],
                "",
            )
        })
        .into_pointer_value()
    }

    pub fn compile_load_arity(
        &self,
        builder: &inkwell::builder::Builder<'c>,
        closure: inkwell::values::PointerValue<'c>,
    ) -> inkwell::values::IntValue<'c> {
        builder
            .build_load(
                builder
                    .build_bitcast(
                        unsafe {
                            builder.build_gep(
                                closure,
                                &[
                                    self.context.i32_type().const_int(0, false),
                                    self.context.i32_type().const_int(1, false),
                                ],
                                "",
                            )
                        },
                        self.type_compiler
                            .compile_arity()
                            .ptr_type(inkwell::AddressSpace::Generic),
                        "",
                    )
                    .into_pointer_value(),
                "",
            )
            .into_int_value()
    }

    pub fn compile_load_environment(
        &self,
        builder: &inkwell::builder::Builder<'c>,
        closure: inkwell::values::PointerValue<'c>,
    ) -> inkwell::values::BasicValueEnum<'c> {
        builder.build_bitcast(
            unsafe {
                builder.build_gep(
                    closure,
                    &[
                        self.context.i32_type().const_int(0, false),
                        self.context.i32_type().const_int(2, false),
                    ],
                    "",
                )
            },
            self.type_compiler
                .compile_unsized_environment()
                .ptr_type(inkwell::AddressSpace::Generic),
            "",
        )
    }

    pub fn compile_store_closure_content(
        &self,
        builder: Arc<inkwell::builder::Builder<'c>>,
        closure_pointer: inkwell::values::PointerValue<'c>,
        entry_function: inkwell::values::FunctionValue<'c>,
        environment_values: &[inkwell::values::BasicValueEnum<'c>],
    ) -> Result<(), CompileError> {
        let environment_type = self
            .type_compiler
            .compile_raw_environment(environment_values.iter().map(|value| value.get_type()));

        let closure = builder
            .build_insert_value(
                self.type_compiler
                    .compile_raw_closure(entry_function.get_type(), environment_type)
                    .get_undef(),
                entry_function.as_global_value().as_pointer_value(),
                0,
                "",
            )
            .unwrap();

        let closure = builder
            .build_insert_value(
                closure,
                self.type_compiler.compile_arity().const_int(
                    utilities::get_arity(entry_function.get_type()) as u64,
                    false,
                ),
                1,
                "",
            )
            .unwrap();

        let closure = builder
            .build_insert_value(
                closure,
                {
                    let mut environment = environment_type.get_undef();

                    for (index, value) in environment_values.iter().copied().enumerate() {
                        environment = builder
                            .build_insert_value(environment, value, index as u32, "")
                            .unwrap()
                            .into_struct_value();
                    }

                    environment
                },
                2,
                "",
            )
            .unwrap();

        builder.build_store(
            builder
                .build_bitcast(
                    closure_pointer,
                    closure
                        .into_struct_value()
                        .get_type()
                        .ptr_type(inkwell::AddressSpace::Generic),
                    "",
                )
                .into_pointer_value(),
            closure,
        );

        Ok(())
    }
}
