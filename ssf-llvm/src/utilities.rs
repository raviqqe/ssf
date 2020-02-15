pub fn get_any_type_enum_undef<'a, 'c>(
    type_: &'a inkwell::types::AnyTypeEnum<'c>,
) -> Option<Box<dyn 'a + inkwell::values::BasicValue<'c>>> {
    match type_ {
        inkwell::types::AnyTypeEnum::ArrayType(type_) => Some(Box::new(type_.get_undef())),
        inkwell::types::AnyTypeEnum::IntType(type_) => Some(Box::new(type_.get_undef())),
        inkwell::types::AnyTypeEnum::FloatType(type_) => Some(Box::new(type_.get_undef())),
        inkwell::types::AnyTypeEnum::FunctionType(_) => None,
        inkwell::types::AnyTypeEnum::PointerType(type_) => Some(Box::new(type_.get_undef())),
        inkwell::types::AnyTypeEnum::StructType(type_) => Some(Box::new(type_.get_undef())),
        inkwell::types::AnyTypeEnum::VectorType(type_) => Some(Box::new(type_.get_undef())),
        inkwell::types::AnyTypeEnum::VoidType(_) => None,
    }
}
