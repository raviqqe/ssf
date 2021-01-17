use super::types;

pub fn compile_declaration(
    module_builder: &fmm::build::ModuleBuilder,
    declaration: &ssf::ir::Declaration,
) {
    module_builder.declare_variable(
        declaration.name(),
        types::compile_unsized_closure(declaration.type_()),
    );
}
