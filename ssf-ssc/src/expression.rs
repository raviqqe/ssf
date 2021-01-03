pub fn compile(_expression: &ssf::ir::Expression) -> (Vec<ssc::ir::Statement>, ssc::ir::Variable) {
    todo!()
}

pub fn compile_arity(arity: u64) -> ssc::ir::Primitive {
    ssc::ir::Primitive::PointerInteger(arity)
}
