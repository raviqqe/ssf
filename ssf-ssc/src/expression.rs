pub fn compile_arity(arity: u64) -> ssc::ir::Primitive {
    ssc::ir::Primitive::PointerInteger(arity)
}
