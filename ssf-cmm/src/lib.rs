mod closures;
mod declarations;
mod definitions;
mod entry_functions;
mod expressions;
mod foreign_declarations;
mod function_applications;
mod instruction_result;
mod instructions;
mod names;
mod types;

use declarations::compile_declaration;
use definitions::compile_definition;
use foreign_declarations::compile_foreign_declaration;

pub fn compile(source_module: &ssf::ir::Module) -> cmm::ir::Module {
    let mut module = cmm::ir::Module::new(vec![], vec![], vec![], vec![]);

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
