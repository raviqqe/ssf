use std::sync::Arc;

pub const FUNCTION_ARGUMENT_OFFSET: usize = 3;

pub fn add_function_to_module<'c>(
    module: Arc<inkwell::module::Module<'c>>,
    name: &str,
    type_: inkwell::types::FunctionType<'c>,
) -> inkwell::values::FunctionValue<'c> {
    module.add_function(name, type_, Some(inkwell::module::Linkage::Private))
}

pub fn get_arity(function_type: inkwell::types::FunctionType) -> usize {
    function_type.count_param_types() as usize - FUNCTION_ARGUMENT_OFFSET
}
