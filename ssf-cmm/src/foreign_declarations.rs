use crate::expressions;
use crate::names;
use crate::types::{self, FUNCTION_ARGUMENT_OFFSET};

pub fn compile_foreign_declaration(
    module: &cmm::ir::Module,
    declaration: &ssf::ir::ForeignDeclaration,
) -> cmm::ir::Module {
    let closure_type = types::compile_unsized_closure(declaration.type_());

    let entry_function_definition = compile_entry_function(declaration);

    cmm::ir::Module::new(
        module.variable_declarations().to_vec(),
        module
            .function_declarations()
            .iter()
            .cloned()
            .chain(vec![cmm::ir::FunctionDeclaration::new(
                declaration.foreign_name(),
                types::compile_foreign_function(declaration.type_()),
            )])
            .collect(),
        module
            .variable_definitions()
            .iter()
            .cloned()
            .chain(vec![cmm::ir::VariableDefinition::new(
                declaration.name(),
                cmm::ir::Record::new(
                    closure_type.clone(),
                    vec![
                        cmm::ir::Variable::new(entry_function_definition.name()).into(),
                        expressions::compile_arity(
                            declaration.type_().arguments().into_iter().count() as u64,
                        )
                        .into(),
                        cmm::ir::Expression::Undefined,
                    ],
                ),
                closure_type,
                true,
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
) -> cmm::ir::FunctionDefinition {
    let arguments = vec![]
        .into_iter()
        .chain(
            declaration
                .type_()
                .arguments()
                .into_iter()
                .enumerate()
                .map(|(index, type_)| {
                    cmm::ir::Argument::new(format!("arg_{}", index), types::compile(type_))
                }),
        )
        .collect::<Vec<_>>();

    let name = names::generate_name();

    cmm::ir::FunctionDefinition::new(
        format!("{}_entry", declaration.name()),
        arguments.clone(),
        vec![
            cmm::ir::Call::new(
                cmm::ir::Variable::new(declaration.foreign_name()),
                arguments
                    .iter()
                    .skip(FUNCTION_ARGUMENT_OFFSET)
                    .map(|argument| cmm::ir::Variable::new(argument.name()).into())
                    .collect(),
                &name,
            )
            .into(),
            cmm::ir::Return::new(cmm::ir::Variable::new(name)).into(),
        ],
        types::compile(declaration.type_().last_result()),
    )
}
