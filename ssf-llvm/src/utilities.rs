pub fn get_arity(function_type: inkwell::types::FunctionType) -> usize {
    function_type.count_param_types() as usize - 1
}
