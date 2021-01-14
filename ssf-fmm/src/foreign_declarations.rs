use super::utilities;
use crate::expressions;
use crate::types::{self, FUNCTION_ARGUMENT_OFFSET};

pub fn compile_foreign_declaration(
    module: &fmm::ir::Module,
    declaration: &ssf::ir::ForeignDeclaration,
) -> fmm::ir::Module {
    let closure_type = types::compile_unsized_closure(declaration.type_());
    let entry_function_definition = compile_entry_function(declaration);

    fmm::ir::Module::new(
        module.variable_declarations().to_vec(),
        module
            .function_declarations()
            .iter()
            .cloned()
            .chain(vec![fmm::ir::FunctionDeclaration::new(
                declaration.foreign_name(),
                types::compile_foreign_function(declaration.type_()),
            )])
            .collect(),
        module
            .variable_definitions()
            .iter()
            .cloned()
            .chain(vec![fmm::ir::VariableDefinition::new(
                declaration.name(),
                fmm::ir::Record::new(
                    closure_type.clone(),
                    vec![
                        fmm::ir::Variable::new(entry_function_definition.name()).into(),
                        expressions::compile_arity(
                            declaration.type_().arguments().into_iter().count() as u64,
                        )
                        .into(),
                        fmm::ir::Undefined::new(types::compile_unsized_environment()).into(),
                    ],
                ),
                closure_type,
                false,
            )])
            .collect(),
        module
            .function_definitions()
            .iter()
            .cloned()
            .chain(vec![entry_function_definition])
            .collect(),
    )
}

fn compile_entry_function(
    declaration: &ssf::ir::ForeignDeclaration,
) -> fmm::ir::FunctionDefinition {
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

    let foreign_function_type = types::compile_foreign_function(declaration.type_());

    fmm::ir::FunctionDefinition::new(
        format!("{}_entry", declaration.name()),
        arguments.clone(),
        {
            let state = fmm::build::BlockState::new();

            state.return_(
                state.call(
                    utilities::variable(declaration.foreign_name(), foreign_function_type.clone()),
                    arguments
                        .iter()
                        .skip(FUNCTION_ARGUMENT_OFFSET)
                        .map(|argument| {
                            utilities::variable(argument.name(), argument.type_().clone())
                        })
                        .collect(),
                ),
            )
        },
        foreign_function_type.result().clone(),
    )
}
