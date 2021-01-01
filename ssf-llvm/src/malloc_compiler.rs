use super::compile_configuration::CompileConfiguration;
use std::sync::Arc;

pub struct MallocCompiler<'c> {
    module: Arc<inkwell::module::Module<'c>>,
    compile_configuration: Arc<CompileConfiguration>,
}

impl<'c> MallocCompiler<'c> {
    pub fn new(
        module: Arc<inkwell::module::Module<'c>>,
        compile_configuration: Arc<CompileConfiguration>,
    ) -> Arc<Self> {
        Self {
            module,
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
        type_: inkwell::types::ArrayType<'c>,
    ) -> inkwell::values::PointerValue<'c> {
        self.compile_aggregate_malloc(builder, type_)
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
                            .get_function(&self.compile_configuration.malloc_function_name)
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
