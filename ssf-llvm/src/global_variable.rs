use std::sync::Arc;

#[derive(Clone)]
pub struct GlobalVariable<'c> {
    global_value: inkwell::values::GlobalValue<'c>,
    pointer_type: inkwell::types::PointerType<'c>,
}

impl<'c> GlobalVariable<'c> {
    pub fn new(
        global_value: inkwell::values::GlobalValue<'c>,
        pointer_type: inkwell::types::PointerType<'c>,
    ) -> Self {
        Self {
            global_value,
            pointer_type,
        }
    }

    pub fn global_value(&self) -> inkwell::values::GlobalValue<'c> {
        self.global_value
    }

    pub fn load(
        &self,
        builder: Arc<inkwell::builder::Builder<'c>>,
    ) -> inkwell::values::PointerValue<'c> {
        builder
            .build_bitcast(self.global_value.as_pointer_value(), self.pointer_type, "")
            .into_pointer_value()
    }
}
