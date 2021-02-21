use fmm::types::{self, Type};

pub fn bitcast(
    builder: &fmm::build::InstructionBuilder,
    argument: impl Into<fmm::build::TypedExpression>,
    to_type: impl Into<Type>,
) -> fmm::build::TypedExpression {
    let argument = argument.into();
    let to_type = to_type.into();

    if argument.type_() == &to_type {
        argument
    } else {
        builder.deconstruct_union(
            fmm::ir::Union::new(
                types::Union::new(vec![argument.type_().clone(), to_type]),
                0,
                argument.expression().clone(),
            ),
            1,
        )
    }
}
