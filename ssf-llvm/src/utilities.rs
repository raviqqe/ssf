pub fn get_any_type_undef<'a, 'c>(
    type_: &'a inkwell::types::AnyTypeEnum<'c>,
) -> Option<inkwell::values::BasicValueEnum<'c>> {
    match type_ {
        inkwell::types::AnyTypeEnum::ArrayType(type_) => Some(type_.get_undef().into()),
        inkwell::types::AnyTypeEnum::IntType(type_) => Some(type_.get_undef().into()),
        inkwell::types::AnyTypeEnum::FloatType(type_) => Some(type_.get_undef().into()),
        inkwell::types::AnyTypeEnum::FunctionType(_) => None,
        inkwell::types::AnyTypeEnum::PointerType(type_) => Some(type_.get_undef().into()),
        inkwell::types::AnyTypeEnum::StructType(type_) => Some(type_.get_undef().into()),
        inkwell::types::AnyTypeEnum::VectorType(type_) => Some(type_.get_undef().into()),
        inkwell::types::AnyTypeEnum::VoidType(_) => None,
    }
}
