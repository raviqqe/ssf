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

    Ok(module.write_bitcode_to_memory().as_slice().to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compile_() {
        compile(
            &ssf::ir::Module::new(vec![], vec![]).unwrap(),
            &CompileConfiguration::new("", vec![], None, None),
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
            &CompileConfiguration::new("", vec![], None, None),
        )
        .unwrap();
    }

    #[test]
    fn compile_with_custom_malloc_function() {
        let algebraic_type =
            ssf::types::Algebraic::new(vec![ssf::types::Constructor::boxed(vec![
                ssf::types::Primitive::Float64.into(),
                ssf::types::Primitive::Float64.into(),
            ])]);

        compile(
            &ssf::ir::Module::new(
                vec![],
                vec![ssf::ir::FunctionDefinition::new(
                    "f",
                    vec![ssf::ir::Argument::new("x", ssf::types::Primitive::Float64)],
                    ssf::ir::ConstructorApplication::new(
                        ssf::ir::Constructor::new(algebraic_type.clone(), 0),
                        vec![42.0.into(), 42.0.into()],
                    ),
                    algebraic_type,
                )
                .into()],
            )
            .unwrap(),
            &CompileConfiguration::new("", vec![], Some("custom_malloc".into()), None),
        )
        .unwrap();
    }

    #[test]
    fn compile_with_panic_function() {
        compile(
            &ssf::ir::Module::new(
                vec![],
                vec![ssf::ir::FunctionDefinition::new(
                    "f",
                    vec![ssf::ir::Argument::new("x", ssf::types::Primitive::Float64)],
                    ssf::ir::PrimitiveCase::new(
                        ssf::ir::Variable::new("x"),
                        vec![ssf::ir::PrimitiveAlternative::new(42.0, 42.0)],
                        None,
                    ),
                    ssf::types::Primitive::Float64,
                )
                .into()],
            )
            .unwrap(),
            &CompileConfiguration::new("", vec![], None, Some("panic".into())),
        )
        .unwrap();
    }

    #[test]
    fn compile_unboxed_constructor() {
        let algebraic_type =
            ssf::types::Algebraic::new(vec![ssf::types::Constructor::unboxed(vec![
                ssf::types::Primitive::Float64.into(),
                ssf::types::Primitive::Float64.into(),
            ])]);

        compile(
            &ssf::ir::Module::new(
                vec![],
                vec![ssf::ir::FunctionDefinition::new(
                    "f",
                    vec![ssf::ir::Argument::new("x", ssf::types::Primitive::Float64)],
                    ssf::ir::ConstructorApplication::new(
                        ssf::ir::Constructor::new(algebraic_type.clone(), 0),
                        vec![42.0.into(), 42.0.into()],
                    ),
                    algebraic_type,
                )
                .into()],
            )
            .unwrap(),
            &CompileConfiguration::new("", vec![], None, None),
        )
        .unwrap();
    }

    #[test]
    fn compile_float_global_variable_reference() {
        compile(
            &ssf::ir::Module::new(
                vec![],
                vec![
                    ssf::ir::ValueDefinition::new("x", 42.0, ssf::types::Primitive::Float64).into(),
                    ssf::ir::ValueDefinition::new(
                        "y",
                        ssf::ir::Variable::new("x"),
                        ssf::types::Primitive::Float64,
                    )
                    .into(),
                ],
            )
            .unwrap(),
            &CompileConfiguration::new("", vec![], None, None),
        )
        .unwrap();
    }

    #[test]
    fn compile_integer_global_variable_reference() {
        compile(
            &ssf::ir::Module::new(
                vec![],
                vec![
                    ssf::ir::ValueDefinition::new("x", 42, ssf::types::Primitive::Integer64).into(),
                    ssf::ir::ValueDefinition::new(
                        "y",
                        ssf::ir::Variable::new("x"),
                        ssf::types::Primitive::Integer64,
                    )
                    .into(),
                ],
            )
            .unwrap(),
            &CompileConfiguration::new("", vec![], None, None),
        )
        .unwrap();
    }

    #[test]
    fn compile_algebraic_global_variable_reference() {
        let algebraic_type =
            ssf::types::Algebraic::new(vec![ssf::types::Constructor::unboxed(vec![])]);

        compile(
            &ssf::ir::Module::new(
                vec![],
                vec![
                    ssf::ir::ValueDefinition::new(
                        "x",
                        ssf::ir::ConstructorApplication::new(
                            ssf::ir::Constructor::new(algebraic_type.clone(), 0),
                            vec![],
                        ),
                        algebraic_type.clone(),
                    )
                    .into(),
                    ssf::ir::ValueDefinition::new("y", ssf::ir::Variable::new("x"), algebraic_type)
                        .into(),
                ],
            )
            .unwrap(),
            &CompileConfiguration::new("", vec![], None, None),
        )
        .unwrap();
    }

    #[test]
    fn compile_recursive_field_access_of_algebraic_types() {
        let algebraic_type =
            ssf::types::Algebraic::new(vec![ssf::types::Constructor::boxed(vec![
                ssf::types::Value::Index(0).into(),
            ])]);

        compile(
            &ssf::ir::Module::new(
                vec![],
                vec![ssf::ir::FunctionDefinition::new(
                    "f",
                    vec![ssf::ir::Argument::new("x", algebraic_type.clone())],
                    ssf::ir::AlgebraicCase::new(
                        ssf::ir::Variable::new("x"),
                        vec![ssf::ir::AlgebraicAlternative::new(
                            ssf::ir::Constructor::new(algebraic_type.clone(), 0),
                            vec!["y".into()],
                            ssf::ir::Variable::new("y"),
                        )],
                        None,
                    ),
                    algebraic_type,
                )
                .into()],
            )
            .unwrap(),
            &CompileConfiguration::new("", vec![], None, None),
        )
        .unwrap();
    }

    #[test]
    fn compile_constructor_applications_of_recursive_algebraic_types() {
        let algebraic_type =
            ssf::types::Algebraic::new(vec![ssf::types::Constructor::boxed(vec![
                ssf::types::Value::Index(0).into(),
            ])]);

        compile(
            &ssf::ir::Module::new(
                vec![],
                vec![ssf::ir::FunctionDefinition::new(
                    "f",
                    vec![ssf::ir::Argument::new("x", algebraic_type.clone())],
                    ssf::ir::ConstructorApplication::new(
                        ssf::ir::Constructor::new(algebraic_type.clone(), 0),
                        vec![ssf::ir::Variable::new("x").into()],
                    ),
                    algebraic_type,
                )
                .into()],
            )
            .unwrap(),
            &CompileConfiguration::new("", vec![], None, None),
        )
        .unwrap();
    }
}
