use fmm::types::{self, Type};

pub fn bitcast(
    state: &fmm::build::BlockState,
    argument: impl Into<fmm::build::TypedExpression>,
    to_type: impl Into<Type>,
) -> fmm::build::TypedExpression {
    let argument = argument.into();
    let from_type = argument.type_();

    state.deconstruct_union(
        fmm::build::TypedExpression::new(
            fmm::ir::Union::new(
                types::Union::new(vec![from_type.clone(), to_type.into()]),
                0,
                argument.expression().clone(),
            ),
            from_type.clone(),
        ),
        1,
    )
}

pub fn variable(
    name: impl Into<String>,
    type_: impl Into<fmm::types::Type>,
) -> fmm::build::TypedExpression {
    fmm::build::TypedExpression::new(fmm::ir::Variable::new(name), type_)
}
