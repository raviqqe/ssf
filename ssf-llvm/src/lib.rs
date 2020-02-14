mod error;
mod expression_compiler;
mod function_compiler;
mod initializer_configuration;
mod module_compiler;
mod type_compiler;
mod utilities;

pub use error::CompileError;
pub use initializer_configuration::InitializerConfiguration;
use module_compiler::ModuleCompiler;
use type_compiler::TypeCompiler;

pub fn compile(
    ir_module: &ssf::ir::Module,
    initializer_configuration: &InitializerConfiguration,
) -> Result<Vec<u8>, CompileError> {
    let context = inkwell::context::Context::create();
    let module = context.create_module("main");
    let type_compiler = TypeCompiler::new(&context);

    ModuleCompiler::new(&context, &module, &type_compiler, initializer_configuration)
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
            &InitializerConfiguration::new("foo", vec![]),
        )
        .unwrap();
    }

    #[test]
    fn compile_with_global_variable() {
        compile(
            &ssf::ir::Module::new(
                vec![],
                vec![ssf::ir::ValueDefinition::new(
                    "foo",
                    ssf::ir::Expression::Number(42.0),
                    ssf::types::Value::Number,
                )
                .into()],
            )
            .unwrap(),
            &InitializerConfiguration::new("foo", vec![]),
        )
        .unwrap();
    }
}
