use crate::expressions;
use crate::types::{self, FUNCTION_ARGUMENT_OFFSET};

pub fn compile_foreign_declaration(
    module_builder: &fmm::build::ModuleBuilder,
    declaration: &ssf::ir::ForeignDeclaration,
) {
    module_builder.define_variable(
        declaration.name(),
        fmm::build::record(vec![
            compile_entry_function(module_builder, declaration),
            expressions::compile_arity(declaration.type_().arguments().into_iter().count()).into(),
            fmm::ir::Undefined::new(types::compile_unsized_environment()).into(),
        ]),
        false,
        false,
    );
}

fn compile_entry_function(
    module_builder: &fmm::build::ModuleBuilder,
    declaration: &ssf::ir::ForeignDeclaration,
) -> fmm::build::TypedExpression {
    let arguments = vec![fmm::ir::Argument::new(
        "_env",
        fmm::types::Pointer::new(types::compile_unsized_environment()),
    )]
    .into_iter()
    .chain(
        declaration
            .type_()
            .arguments()
            .into_iter()
            .enumerate()
            .map(|(index, type_)| {
                fmm::ir::Argument::new(format!("arg_{}", index), types::compile(type_))
            }),
    )
    .collect::<Vec<_>>();

    let foreign_function_type = types::compile_foreign_function_of_declaration(declaration.type_());

    module_builder.define_anonymous_function(
        arguments.clone(),
        |instruction_builder| {
            instruction_builder.return_(
                instruction_builder.call(
                    module_builder.declare_function(
                        declaration.foreign_name(),
                        foreign_function_type.clone(),
                    ),
                    arguments
                        .iter()
                        .skip(FUNCTION_ARGUMENT_OFFSET)
                        .map(|argument| {
                            fmm::build::variable(argument.name(), argument.type_().clone())
                        })
                        .collect(),
                ),
            )
        },
        foreign_function_type.result().clone(),
        fmm::types::CallingConvention::Source,
    )
}
