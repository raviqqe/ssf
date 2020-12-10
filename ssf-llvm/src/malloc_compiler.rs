use super::compile_configuration::CompileConfiguration;
use super::type_compiler::TypeCompiler;
use inkwell::types::BasicType;
use std::sync::Arc;

pub struct MallocCompiler<'c> {
    module: Arc<inkwell::module::Module<'c>>,
    type_compiler: Arc<TypeCompiler<'c>>,
    compile_configuration: Arc<CompileConfiguration>,
}

impl<'c> MallocCompiler<'c> {
    pub fn new(
        module: Arc<inkwell::module::Module<'c>>,
        type_compiler: Arc<TypeCompiler<'c>>,
        compile_configuration: Arc<CompileConfiguration>,
    ) -> Arc<Self> {
        Self {
            module,
            type_compiler,
            compile_configuration,
        }
        .into()
    }

    pub fn compile_struct_malloc(
        &self,
        builder: &inkwell::builder::Builder<'c>,
        type_: inkwell::types::StructType<'c>,
    ) -> inkwell::values::PointerValue<'c> {
        self.compile_aggregate_malloc(builder, type_)
    }

    pub fn compile_array_malloc(
        &self,
        builder: &inkwell::builder::Builder<'c>,
        element_type: inkwell::types::BasicTypeEnum<'c>,
        length: inkwell::values::IntValue<'c>,
    ) -> inkwell::values::PointerValue<'c> {
        builder
            .build_bitcast(
                builder
                    .build_call(
                        self.module
                            .get_function(self.compile_configuration.malloc_function_name())
                            .unwrap(),
                        &[builder
                            .build_int_mul(
                                length.get_type().const_int(
                                    self.type_compiler.get_store_size(element_type),
                                    false,
                                ),
                                length,
                                "",
                            )
                            .into()],
                        "",
                    )
                    .try_as_basic_value()
                    .left()
                    .unwrap(),
                element_type.ptr_type(inkwell::AddressSpace::Generic),
                "",
            )
            .into_pointer_value()
    }

    fn compile_aggregate_malloc(
        &self,
        builder: &inkwell::builder::Builder<'c>,
        type_: impl inkwell::types::BasicType<'c>,
    ) -> inkwell::values::PointerValue<'c> {
        builder
            .build_bitcast(
                builder
                    .build_call(
                        self.module
                            .get_function(self.compile_configuration.malloc_function_name())
                            .unwrap(),
                        &[type_.size_of().unwrap().into()],
                        "",
                    )
                    .try_as_basic_value()
                    .left()
                    .unwrap(),
                type_.ptr_type(inkwell::AddressSpace::Generic),
                "",
            )
            .into_pointer_value()
    }
}
