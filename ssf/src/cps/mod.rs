use crate::ir::*;

pub fn transform_to_cps(module: &Module) -> Module {
    Module::new(
        module.foreign_declarations().to_vec(),
        module.declarations().to_vec(),
        module
            .definitions()
            .iter()
            .map(|definition| {
                Definition::with_options(
                    definition.name(),
                    definition.environment().to_vec(),
                    definition
                        .arguments()
                        .iter()
                        .cloned()
                        .chain(vec![Argument::new(CONTINUATION_NAME, "foo")]),
                    transform_expression(definition.body())(),
                    definition.result_type().clone(),
                    definition.is_thunk(),
                )
            })
            .collect(),
    )
}

fn transform_expression(
    expression: &Expression,
    continuation: impl Into<Continuation>,
) -> Expression {
    match expression {
        Expression::Bitcast(bitcast) => bitcast.clone().into(),
        Expression::FunctionApplication(application) => transform_expression(
            application.first_function().clone(),
            application.arguments().into_iter().cloned().collect(),
            continuation.into(),
        )
        .clone()
        .into(),
        Expression::Primitive(primitive) => primitive.clone().into(),
        Expression::Variable(variable) => variable.clone().into(),
        _ => todo!(),
    }
}
