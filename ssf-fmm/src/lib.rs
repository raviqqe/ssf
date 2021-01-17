mod closures;
mod declarations;
mod definitions;
mod entry_functions;
mod expressions;
mod foreign_declarations;
mod function_applications;
mod types;
mod utilities;

use declarations::compile_declaration;
use definitions::compile_definition;
use foreign_declarations::compile_foreign_declaration;

pub fn compile(source_module: &ssf::ir::Module) -> fmm::ir::Module {
    let module_builder = fmm::build::ModuleBuilder::new();

    for declaration in source_module.foreign_declarations() {
        compile_foreign_declaration(&module_builder, declaration);
    }

    for declaration in source_module.declarations() {
        compile_declaration(&module_builder, declaration);
    }

    for definition in source_module.definitions() {
        compile_definition(&module_builder, definition);
    }

    module_builder.as_module()
}
