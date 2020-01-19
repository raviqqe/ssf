pub fn get_any_type_enum_undef<'a, 'c>(
    type_: &'a inkwell::types::AnyTypeEnum<'c>,
) -> Box<dyn 'a + inkwell::values::BasicValue<'c>> {
    match type_ {
        inkwell::types::AnyTypeEnum::ArrayType(type_) => Box::new(type_.get_undef()),
        inkwell::types::AnyTypeEnum::IntType(type_) => Box::new(type_.get_undef()),
        inkwell::types::AnyTypeEnum::FloatType(type_) => Box::new(type_.get_undef()),
        inkwell::types::AnyTypeEnum::FunctionType(_) => unreachable!(),
        inkwell::types::AnyTypeEnum::PointerType(type_) => Box::new(type_.get_undef()),
        inkwell::types::AnyTypeEnum::StructType(type_) => Box::new(type_.get_undef()),
        inkwell::types::AnyTypeEnum::VectorType(type_) => Box::new(type_.get_undef()),
        inkwell::types::AnyTypeEnum::VoidType(_) => unreachable!(),
    }
}
