use super::types;

pub fn compile_declaration(
    module: &ssc::ir::Module,
    declaration: &ssf::ir::Declaration,
) -> ssc::ir::Module {
    ssc::ir::Module::new(
        module
            .variable_declarations()
            .iter()
            .cloned()
            .chain(vec![ssc::ir::VariableDeclaration::new(
                declaration.name(),
                types::compile_unsized_closure(declaration.type_()),
            )])
            .collect(),
        module.function_declarations().to_vec(),
        module.variable_definitions().to_vec(),
        module.function_definitions().to_vec(),
    )
}
