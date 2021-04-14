use fmm::types::{self, Type};

pub fn bit_cast(
    builder: &fmm::build::InstructionBuilder,
    argument: impl Into<fmm::build::TypedExpression>,
    to_type: impl Into<Type>,
) -> Result<fmm::build::TypedExpression, fmm::build::BuildError> {
    let argument = argument.into();
    let to_type = to_type.into();

    Ok(if argument.type_() == &to_type {
        argument
    } else {
        builder.deconstruct_union(
            fmm::ir::Union::new(
                types::Union::new(vec![argument.type_().clone(), to_type]),
                0,
                argument.expression().clone(),
            ),
            1,
        )?
    })
}
