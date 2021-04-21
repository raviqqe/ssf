use super::expressions;
use super::types;

pub fn compile_load_entry_pointer(
    builder: &fmm::build::InstructionBuilder,
    closure_pointer: impl Into<fmm::build::TypedExpression>,
) -> Result<fmm::build::TypedExpression, fmm::build::BuildError> {
    // Entry functions of thunks need to be loaded atomically
    // to make thunk update thread-safe.
    builder.atomic_load(builder.record_address(closure_pointer, 0)?)
}

pub fn compile_load_arity(
    builder: &fmm::build::InstructionBuilder,
    closure_pointer: impl Into<fmm::build::TypedExpression>,
) -> Result<fmm::build::TypedExpression, fmm::build::BuildError> {
    builder.load(builder.record_address(closure_pointer, 1)?)
}

pub fn compile_environment_pointer(
    builder: &fmm::build::InstructionBuilder,
    closure_pointer: impl Into<fmm::build::TypedExpression>,
) -> Result<fmm::build::TypedExpression, fmm::build::BuildError> {
    Ok(fmm::build::bit_cast(
        fmm::types::Pointer::new(types::compile_unsized_environment()),
        builder.record_address(closure_pointer, 2)?,
    )
    .into())
}

pub fn compile_closure_content(
    entry_function: impl Into<fmm::build::TypedExpression>,
    free_variables: Vec<fmm::build::TypedExpression>,
) -> fmm::build::TypedExpression {
    let entry_function = entry_function.into();

    fmm::build::record(vec![
        entry_function.clone(),
        expressions::compile_arity(types::get_arity(
            entry_function.type_().to_function().unwrap(),
        ))
        .into(),
        fmm::build::record(free_variables).into(),
    ])
    .into()
}
