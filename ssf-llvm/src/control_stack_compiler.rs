use super::compile_configuration::CompileConfiguration;
use super::type_compiler::TypeCompiler;
use inkwell::types::BasicType;
use std::sync::Arc;

pub struct ControlStackCompiler<'c> {
    context: &'c inkwell::context::Context,
    module: Arc<inkwell::module::Module<'c>>,
    type_compiler: Arc<TypeCompiler<'c>>,
    compile_configuration: Arc<CompileConfiguration>,
}

impl<'c> ControlStackCompiler<'c> {
    pub fn new(
        context: &'c inkwell::context::Context,
        module: Arc<inkwell::module::Module<'c>>,
        type_compiler: Arc<TypeCompiler<'c>>,
        compile_configuration: Arc<CompileConfiguration>,
    ) -> Arc<Self> {
        Self {
            context,
            module,
            type_compiler,
            compile_configuration,
        }
        .into()
    }

    pub fn compile_push(
        &self,
        builder: &inkwell::builder::Builder<'c>,
        stack: inkwell::values::StructValue<'c>,
        values: &[inkwell::values::BasicValueEnum<'c>],
    ) -> inkwell::values::StructValue<'c> {
        let pointer = builder
            .build_extract_value(stack, 0, "")
            .unwrap()
            .into_pointer_value();
        let length = builder
            .build_extract_value(stack, 1, "")
            .unwrap()
            .into_int_value();
        let capacity = builder
            .build_extract_value(stack, 2, "")
            .unwrap()
            .into_int_value();

        let realloc_block = self.append_basic_block(builder, "realloc");
        let push_block = self.append_basic_block(builder, "push");

        let total_size: usize = values
            .iter()
            .map(|value| self.type_compiler.calculate_aligned_size(value.get_type()))
            .sum();

        builder.build_conditional_branch(
            builder.build_int_compare(
                inkwell::IntPredicate::ULT,
                capacity,
                builder.build_int_add(
                    length,
                    length.get_type().const_int(total_size as u64, false),
                    "",
                ),
                "",
            ),
            realloc_block,
            push_block,
        );

        builder.position_at_end(realloc_block);

        let new_capacity =
            builder.build_int_mul(capacity, capacity.get_type().const_int(2, false), "");
        let stack = builder
            .build_insert_value(
                stack,
                self.compile_realloc(builder, pointer, new_capacity),
                0,
                "",
            )
            .unwrap()
            .into_struct_value();
        let stack = builder
            .build_insert_value(
                stack,
                builder.build_int_add(
                    length,
                    length.get_type().const_int(total_size as u64, false),
                    "",
                ),
                2,
                "",
            )
            .unwrap()
            .into_struct_value();
        let stack = builder
            .build_insert_value(stack, new_capacity, 2, "")
            .unwrap()
            .into_struct_value();

        builder.build_unconditional_branch(push_block);

        builder.position_at_end(push_block);

        for &value in values {
            let size = self.type_compiler.calculate_aligned_size(value.get_type());
            let pointer_int_type = self.type_compiler.compile_pointer_sized_integer();

            builder.build_store(
                builder.build_int_to_ptr(
                    builder.build_int_add(
                        builder.build_ptr_to_int(pointer, pointer_int_type, ""),
                        pointer_int_type.const_int(size as u64, false),
                        "",
                    ),
                    value.get_type().ptr_type(inkwell::AddressSpace::Generic),
                    "",
                ),
                value,
            );
        }

        builder
            .build_insert_value(stack, length, 2, "")
            .unwrap()
            .into_struct_value();

        stack
    }

    pub fn compile_pop(
        &self,
        builder: &inkwell::builder::Builder<'c>,
    ) -> Vec<inkwell::values::BasicValueEnum<'c>> {
        todo!()
    }

    fn compile_realloc(
        &self,
        builder: &inkwell::builder::Builder<'c>,
        pointer: inkwell::values::PointerValue<'c>,
        size: inkwell::values::IntValue<'c>,
    ) -> inkwell::values::PointerValue<'c> {
        builder
            .build_call(
                self.module
                    .get_function(&self.compile_configuration.realloc_function_name)
                    .unwrap(),
                &[pointer.into(), size.into()],
                "",
            )
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_pointer_value()
    }

    fn append_basic_block(
        &self,
        builder: &inkwell::builder::Builder<'c>,
        name: &str,
    ) -> inkwell::basic_block::BasicBlock<'c> {
        self.context.append_basic_block(
            builder.get_insert_block().unwrap().get_parent().unwrap(),
            name,
        )
    }
}
