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

pub fn variable(
    name: impl Into<String>,
    type_: impl Into<fmm::types::Type>,
) -> fmm::build::TypedExpression {
    fmm::build::TypedExpression::new(fmm::ir::Variable::new(name), type_)
}

pub fn record(elements: Vec<fmm::build::TypedExpression>) -> fmm::ir::Record {
    fmm::ir::Record::new(
        fmm::types::Record::new(
            elements
                .iter()
                .map(|element| element.type_().clone())
                .collect(),
        ),
        elements
            .iter()
            .map(|element| element.expression().clone())
            .collect(),
    )
}
