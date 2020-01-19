mod error;
mod expression_compiler;
mod function_compiler;
mod initializer_configuration;
mod initializer_sorter;
mod module_compiler;
mod type_compiler;

pub use error::CompileError;
pub use initializer_configuration::InitializerConfiguration;
use module_compiler::ModuleCompiler;
use type_compiler::TypeCompiler;

pub fn compile(
    ast_module: &ssf::ast::Module,
    initializer_configuration: &InitializerConfiguration,
) -> Result<Vec<u8>, CompileError> {
    ssf::verify(ast_module)?;

    let context = inkwell::context::Context::create();
    let module = context.create_module("main");
    let type_compiler = TypeCompiler::new(&context, &module);

    ModuleCompiler::new(&context, &module, &type_compiler, initializer_configuration)
        .compile(ast_module)?;

    let bitcode = module.write_bitcode_to_memory().as_slice().to_vec();

    Ok(bitcode)
}
