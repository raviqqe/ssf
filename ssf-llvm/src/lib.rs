mod compile_configuration;
mod error;
mod expression_compiler;
mod function_compiler;
mod module_compiler;
mod type_compiler;
mod utilities;

pub use compile_configuration::CompileConfiguration;
pub use error::CompileError;
use module_compiler::ModuleCompiler;
use type_compiler::TypeCompiler;

pub fn compile(
    ir_module: &ssf::ir::Module,
    compile_configuration: &CompileConfiguration,
) -> Result<Vec<u8>, CompileError> {
    let context = inkwell::context::Context::create();
    let module = context.create_module("main");
    let type_compiler = TypeCompiler::new(&context);

    ModuleCompiler::new(&context, &module, &type_compiler, compile_configuration)
        .compile(ir_module)?;

    let bitcode = module.write_bitcode_to_memory().as_slice().to_vec();

    Ok(bitcode)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compile_() {
        compile(
            &ssf::ir::Module::new(vec![], vec![]).unwrap(),
            &CompileConfiguration::new("foo", vec![], None, None),
        )
        .unwrap();
    }

    #[test]
    fn compile_with_global_variable() {
        compile(
            &ssf::ir::Module::new(
                vec![],
                vec![
                    ssf::ir::ValueDefinition::new("foo", 42.0, ssf::types::Primitive::Float64)
                        .into(),
                ],
            )
            .unwrap(),
            &CompileConfiguration::new("foo", vec![], None, None),
        )
        .unwrap();
    }
}
