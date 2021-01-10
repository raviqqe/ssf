use super::names;
use fmm::build::*;
use fmm::types::{self, Type};

pub fn bitcast(context: impl Into<BuildContext>, to: impl Into<Type>) -> BuildContext {
    let context = context.into();
    let from = context.type_();

    deconstruct_union(
        BuildContext::new(
            context.instructions().to_vec(),
            fmm::ir::Union::new(
                types::Union::new(vec![from.clone(), to.into()]),
                0,
                context.expression().clone(),
            ),
            from.clone(),
        ),
        1,
    )
}

pub fn if_(
    condition: impl Into<BuildContext>,
    then: fmm::ir::Block,
    else_: fmm::ir::Block,
) -> BuildContext {
    let condition = condition.into();
    let name = names::generate_name();
    let type_ = if let Some(branch) = then.terminal_instruction().to_branch() {
        branch.type_().clone()
    } else if let Some(branch) = else_.terminal_instruction().to_branch() {
        branch.type_().clone()
    } else {
        fmm::types::Record::new(vec![]).into()
    };

    BuildContext::new(
        condition
            .instructions()
            .iter()
            .cloned()
            .chain(vec![fmm::ir::If::new(
                type_.clone(),
                condition.expression().clone(),
                then,
                else_,
                names::generate_name(),
            )
            .into()]),
        fmm::ir::Variable::new(name),
        type_,
    )
}

pub fn side_effect<T: IntoIterator<Item = fmm::ir::Instruction>>(
    context: impl Into<BuildContext>,
    instructions: impl Fn(BuildContext) -> T,
) -> BuildContext {
    let context = context.into();

    BuildContext::new(
        context
            .instructions()
            .iter()
            .cloned()
            .chain(instructions(BuildContext::from_expression(
                context.expression().clone(),
                context.type_().clone(),
            ))),
        context.expression().clone(),
        context.type_().clone(),
    )
}
