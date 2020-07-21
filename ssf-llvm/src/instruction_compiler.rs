use inkwell::values::BasicValue;

pub struct InstructionCompiler {}

impl InstructionCompiler {
    pub fn compile_atomic_load<'c>(
        builder: &inkwell::builder::Builder<'c>,
        pointer: inkwell::values::PointerValue<'c>,
    ) -> inkwell::values::BasicValueEnum<'c> {
        let value = builder.build_load(pointer, "");

        value
            .as_instruction_value()
            .unwrap()
            .set_alignment(8)
            .unwrap();
        value
            .as_instruction_value()
            .unwrap()
            .set_atomic_ordering(inkwell::AtomicOrdering::SequentiallyConsistent)
            .unwrap();

        value
    }

    pub fn compile_atomic_store<'c>(
        builder: &inkwell::builder::Builder<'c>,
        pointer: inkwell::values::PointerValue<'c>,
        value: impl inkwell::values::BasicValue<'c>,
    ) {
        let store_value = builder.build_store(pointer, value);
        store_value.set_alignment(8).unwrap();
        store_value
            .set_atomic_ordering(inkwell::AtomicOrdering::SequentiallyConsistent)
            .unwrap();
    }
}
