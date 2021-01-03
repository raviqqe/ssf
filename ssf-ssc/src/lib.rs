mod compile_configuration;
mod declaration;
mod definition;
mod expression;
mod foreign_declaration;
mod types;

use compile_configuration::CompileConfiguration;
use declaration::compile_declaration;
use definition::compile_definition;
use foreign_declaration::compile_foreign_declaration;
use std::sync::Arc;

pub fn compile(
    source_module: &ssf::ir::Module,
    compile_configuration: Arc<CompileConfiguration>,
) -> ssc::ir::Module {
    let mut module = ssc::ir::Module::new(
        vec![],
        vec![ssc::ir::FunctionDeclaration::new(
            &compile_configuration.malloc_function_name,
            ssc::types::Function::new(vec![], types::compile_generic_pointer()),
        )],
        vec![],
        vec![],
    );

    for declaration in source_module.foreign_declarations() {
        module = compile_foreign_declaration(&module, declaration);
    }

    for declaration in source_module.declarations() {
        module = compile_declaration(&module, declaration);
    }

    for definition in source_module.definitions() {
        module = compile_definition(&module, definition);
    }

    module
}
