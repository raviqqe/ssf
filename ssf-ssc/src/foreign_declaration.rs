use super::expression;
use super::type_::{self, FUNCTION_ARGUMENT_OFFSET};

pub fn compile_foreign_declaration(
    module: &ssc::ir::Module,
    declaration: &ssf::ir::ForeignDeclaration,
) -> ssc::ir::Module {
    let closure_type = type_::compile_unsized_closure(declaration.type_());

    let entry_function_definition = compile_entry_function(declaration);

    ssc::ir::Module::new(
        module.variable_declarations().to_vec(),
        module
            .function_declarations()
            .iter()
            .cloned()
            .chain(vec![ssc::ir::FunctionDeclaration::new(
                declaration.foreign_name(),
                type_::compile_foreign_function(declaration.type_()),
            )])
            .collect(),
        module
            .variable_definitions()
            .iter()
            .cloned()
            .chain(vec![ssc::ir::VariableDefinition::new(
                declaration.name(),
                ssc::ir::Constructor::new(
                    closure_type.clone(),
                    vec![
                        ssc::ir::Variable::new(entry_function_definition.name()).into(),
                        expression::compile_arity(
                            declaration.type_().arguments().into_iter().count() as u64,
                        )
                        .into(),
                        ssc::ir::Expression::Undefined,
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
) -> ssc::ir::FunctionDefinition {
    let arguments = vec![]
        .into_iter()
        .chain(
            declaration
                .type_()
                .arguments()
                .into_iter()
                .enumerate()
                .map(|(index, type_)| {
                    ssc::ir::Argument::new(format!("arg_{}", index), type_::compile(type_))
                }),
        )
        .collect::<Vec<_>>();

    ssc::ir::FunctionDefinition::new(
        format!("{}_entry", declaration.name()),
        arguments.clone(),
        vec![ssc::ir::Return::new(ssc::ir::Call::new(
            ssc::ir::Variable::new(declaration.foreign_name()),
            arguments
                .iter()
                .skip(FUNCTION_ARGUMENT_OFFSET)
                .map(|argument| ssc::ir::Variable::new(argument.name()).into())
                .collect(),
        ))
        .into()],
        type_::compile(declaration.type_().last_result()),
    )
}
