use crate::expressions;
use crate::names;
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
) -> fmm::ir::FunctionDefinition {
    let arguments = vec![]
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

    let name = names::generate_name();
    let result_type = types::compile(declaration.type_().last_result());

    fmm::ir::FunctionDefinition::new(
        format!("{}_entry", declaration.name()),
        arguments.clone(),
        fmm::ir::Block::new(
            vec![fmm::ir::Call::new(
                types::compile_foreign_function(declaration.type_()),
                fmm::ir::Variable::new(declaration.foreign_name()),
                arguments
                    .iter()
                    .skip(FUNCTION_ARGUMENT_OFFSET)
                    .map(|argument| fmm::ir::Variable::new(argument.name()).into())
                    .collect(),
                &name,
            )
            .into()],
            fmm::ir::Return::new(result_type.clone(), fmm::ir::Variable::new(name)),
        ),
        result_type,
    )
}
