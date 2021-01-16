use super::types;
use super::utilities;

pub fn compile_load_entry_pointer(
    state: &fmm::build::BlockState,
    closure_pointer: impl Into<fmm::build::TypedExpression>,
) -> fmm::build::TypedExpression {
    // Entry functions of thunks need to be loaded atomically
    // to make thunk update thread-safe.
    state.atomic_load(state.record_address(closure_pointer, 0))
}

pub fn compile_load_arity(
    state: &fmm::build::BlockState,
    closure_pointer: impl Into<fmm::build::TypedExpression>,
) -> fmm::build::TypedExpression {
    state.load(state.record_address(closure_pointer, 1))
}

pub fn compile_environment_pointer(
    state: &fmm::build::BlockState,
    closure_pointer: impl Into<fmm::build::TypedExpression>,
) -> fmm::build::TypedExpression {
    utilities::bitcast(
        state,
        state.record_address(closure_pointer, 2),
        fmm::types::Pointer::new(types::compile_unsized_environment()),
    )
}

pub fn compile_closure_content(
    entry_function: impl Into<fmm::build::TypedExpression>,
    free_variables: Vec<fmm::build::TypedExpression>,
) -> fmm::build::TypedExpression {
    let entry_function = entry_function.into();

    utilities::record(vec![
        entry_function.clone(),
        fmm::ir::Primitive::PointerInteger(types::get_arity(
            entry_function.type_().to_function().unwrap(),
        ) as u64)
        .into(),
        utilities::record(free_variables).into(),
    ])
    .into()
}
