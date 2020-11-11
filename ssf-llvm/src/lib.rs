mod compile_configuration;
mod error;
mod expression_compiler;
mod function_compiler;
mod instruction_compiler;
mod module_compiler;
mod type_compiler;

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
    use lazy_static::lazy_static;

    const NUMBER_ARITHMETIC_OPERATORS: [ssf::ir::Operator; 4] = [
        ssf::ir::Operator::Add,
        ssf::ir::Operator::Subtract,
        ssf::ir::Operator::Multiply,
        ssf::ir::Operator::Divide,
    ];

    const NUMBER_COMPARISON_OPERATORS: [ssf::ir::Operator; 6] = [
        ssf::ir::Operator::Equal,
        ssf::ir::Operator::NotEqual,
        ssf::ir::Operator::GreaterThan,
        ssf::ir::Operator::GreaterThanOrEqual,
        ssf::ir::Operator::LessThan,
        ssf::ir::Operator::LessThanOrEqual,
    ];

    lazy_static! {
        static ref COMPILE_CONFIGURATION: CompileConfiguration =
            CompileConfiguration::new(None, None);
    }

    #[test]
    fn compile_() {
        compile(
            &ssf::ir::Module::new(vec![], vec![]).unwrap(),
            &COMPILE_CONFIGURATION,
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
                vec![ssf::ir::Definition::new(
                    "f",
                    vec![ssf::ir::Argument::new("x", ssf::types::Primitive::Float64)],
                    ssf::ir::ConstructorApplication::new(
                        ssf::ir::Constructor::new(algebraic_type.clone(), 0),
                        vec![42.0.into(), 42.0.into()],
                    ),
                    algebraic_type,
                )],
            )
            .unwrap(),
            &CompileConfiguration::new(Some("custom_malloc".into()), None),
        )
        .unwrap();
    }

    #[test]
    fn compile_with_panic_function() {
        compile(
            &ssf::ir::Module::new(
                vec![],
                vec![ssf::ir::Definition::new(
                    "f",
                    vec![ssf::ir::Argument::new("x", ssf::types::Primitive::Float64)],
                    ssf::ir::PrimitiveCase::new(
                        ssf::types::Primitive::Float64,
                        ssf::ir::Variable::new("x"),
                        vec![ssf::ir::PrimitiveAlternative::new(42.0, 42.0)],
                        None,
                    ),
                    ssf::types::Primitive::Float64,
                )],
            )
            .unwrap(),
            &CompileConfiguration::new(None, Some("panic".into())),
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
                vec![ssf::ir::Definition::new(
                    "f",
                    vec![ssf::ir::Argument::new("x", ssf::types::Primitive::Float64)],
                    ssf::ir::ConstructorApplication::new(
                        ssf::ir::Constructor::new(algebraic_type.clone(), 0),
                        vec![42.0.into(), 42.0.into()],
                    ),
                    algebraic_type,
                )],
            )
            .unwrap(),
            &COMPILE_CONFIGURATION,
        )
        .unwrap();
    }

    #[test]
    fn compile_recursive_field_access_of_algebraic_types() {
        let algebraic_type =
            ssf::types::Algebraic::new(vec![ssf::types::Constructor::boxed(vec![
                ssf::types::Type::Index(0),
            ])]);

        compile(
            &ssf::ir::Module::new(
                vec![],
                vec![ssf::ir::Definition::new(
                    "f",
                    vec![ssf::ir::Argument::new("x", algebraic_type.clone())],
                    ssf::ir::AlgebraicCase::new(
                        algebraic_type.clone(),
                        ssf::ir::Variable::new("x"),
                        vec![ssf::ir::AlgebraicAlternative::new(
                            ssf::ir::Constructor::new(algebraic_type.clone(), 0),
                            vec!["y".into()],
                            ssf::ir::Variable::new("y"),
                        )],
                        None,
                    ),
                    algebraic_type,
                )],
            )
            .unwrap(),
            &COMPILE_CONFIGURATION,
        )
        .unwrap();
    }

    #[test]
    fn compile_constructor_applications_of_recursive_algebraic_types() {
        let algebraic_type =
            ssf::types::Algebraic::new(vec![ssf::types::Constructor::boxed(vec![
                ssf::types::Type::Index(0),
            ])]);

        compile(
            &ssf::ir::Module::new(
                vec![],
                vec![ssf::ir::Definition::new(
                    "f",
                    vec![ssf::ir::Argument::new("x", algebraic_type.clone())],
                    ssf::ir::ConstructorApplication::new(
                        ssf::ir::Constructor::new(algebraic_type.clone(), 0),
                        vec![ssf::ir::Variable::new("x").into()],
                    ),
                    algebraic_type,
                )],
            )
            .unwrap(),
            &COMPILE_CONFIGURATION,
        )
        .unwrap();
    }

    #[test]
    fn compile_algebraic_types_with_custom_tags() {
        let algebraic_type = ssf::types::Algebraic::with_tags(
            vec![(42, ssf::types::Constructor::unboxed(vec![]))]
                .into_iter()
                .collect(),
        );

        compile(
            &ssf::ir::Module::new(
                vec![],
                vec![ssf::ir::Definition::new(
                    "f",
                    vec![ssf::ir::Argument::new("x", algebraic_type.clone())],
                    ssf::ir::AlgebraicCase::new(
                        algebraic_type.clone(),
                        ssf::ir::Variable::new("x"),
                        vec![ssf::ir::AlgebraicAlternative::new(
                            ssf::ir::Constructor::new(algebraic_type, 42),
                            vec![],
                            42.0,
                        )],
                        None,
                    ),
                    ssf::types::Primitive::Float64,
                )],
            )
            .unwrap(),
            &COMPILE_CONFIGURATION,
        )
        .unwrap();
    }

    #[test]
    fn compile_nested_algebraic_cases() {
        let algebraic_type = ssf::types::Algebraic::new(
            vec![ssf::types::Constructor::unboxed(vec![])]
                .into_iter()
                .collect(),
        );

        compile(
            &ssf::ir::Module::new(
                vec![],
                vec![ssf::ir::Definition::new(
                    "f",
                    vec![ssf::ir::Argument::new("x", algebraic_type.clone())],
                    ssf::ir::AlgebraicCase::new(
                        algebraic_type.clone(),
                        ssf::ir::Variable::new("x"),
                        vec![ssf::ir::AlgebraicAlternative::new(
                            ssf::ir::Constructor::new(algebraic_type.clone(), 0),
                            vec![],
                            ssf::ir::AlgebraicCase::new(
                                algebraic_type.clone(),
                                ssf::ir::Variable::new("x"),
                                vec![ssf::ir::AlgebraicAlternative::new(
                                    ssf::ir::Constructor::new(algebraic_type, 0),
                                    vec![],
                                    42.0,
                                )],
                                None,
                            ),
                        )],
                        None,
                    ),
                    ssf::types::Primitive::Float64,
                )],
            )
            .unwrap(),
            &COMPILE_CONFIGURATION,
        )
        .unwrap();
    }

    #[test]
    fn compile_nested_algebraic_cases_with_default_alternatives() {
        let algebraic_type = ssf::types::Algebraic::new(
            vec![ssf::types::Constructor::unboxed(vec![])]
                .into_iter()
                .collect(),
        );

        compile(
            &ssf::ir::Module::new(
                vec![],
                vec![ssf::ir::Definition::new(
                    "f",
                    vec![ssf::ir::Argument::new("x", algebraic_type.clone())],
                    ssf::ir::AlgebraicCase::new(
                        algebraic_type.clone(),
                        ssf::ir::Variable::new("x"),
                        vec![ssf::ir::AlgebraicAlternative::new(
                            ssf::ir::Constructor::new(algebraic_type.clone(), 0),
                            vec![],
                            ssf::ir::AlgebraicCase::new(
                                algebraic_type.clone(),
                                ssf::ir::Variable::new("x"),
                                vec![ssf::ir::AlgebraicAlternative::new(
                                    ssf::ir::Constructor::new(algebraic_type, 0),
                                    vec![],
                                    42.0,
                                )],
                                Some(ssf::ir::DefaultAlternative::new("x", 42.0)),
                            ),
                        )],
                        Some(ssf::ir::DefaultAlternative::new("x", 42.0)),
                    ),
                    ssf::types::Primitive::Float64,
                )],
            )
            .unwrap(),
            &COMPILE_CONFIGURATION,
        )
        .unwrap();
    }

    #[test]
    fn compile_bitcast() {
        compile(
            &ssf::ir::Module::new(
                vec![],
                vec![ssf::ir::Definition::new(
                    "f",
                    vec![ssf::ir::Argument::new("x", ssf::types::Primitive::Float64)],
                    ssf::ir::Bitcast::new(42, ssf::types::Primitive::Float64),
                    ssf::types::Primitive::Float64,
                )],
            )
            .unwrap(),
            &COMPILE_CONFIGURATION,
        )
        .unwrap();
    }

    #[test]
    fn compile_function_applications() {
        compile(
            &ssf::ir::Module::new(
                vec![],
                vec![
                    ssf::ir::Definition::new(
                        "f",
                        vec![ssf::ir::Argument::new("x", ssf::types::Primitive::Float64)],
                        42.0,
                        ssf::types::Primitive::Float64,
                    ),
                    ssf::ir::Definition::new(
                        "g",
                        vec![ssf::ir::Argument::new("x", ssf::types::Primitive::Float64)],
                        ssf::ir::FunctionApplication::new(ssf::ir::Variable::new("f"), 42.0),
                        ssf::types::Primitive::Float64,
                    ),
                ],
            )
            .unwrap(),
            &COMPILE_CONFIGURATION,
        )
        .unwrap();
    }

    #[test]
    fn compile_integer_arithmetic_operators() {
        for operator in &NUMBER_ARITHMETIC_OPERATORS {
            compile(
                &ssf::ir::Module::new(
                    vec![],
                    vec![ssf::ir::Definition::new(
                        "f",
                        vec![ssf::ir::Argument::new("x", ssf::types::Primitive::Float64)],
                        ssf::ir::Operation::new(*operator, 42, 42),
                        ssf::types::Primitive::Integer64,
                    )],
                )
                .unwrap(),
                &COMPILE_CONFIGURATION,
            )
            .unwrap();
        }
    }

    #[test]
    fn compile_float_arithmetic_operators() {
        for operator in &NUMBER_ARITHMETIC_OPERATORS {
            compile(
                &ssf::ir::Module::new(
                    vec![],
                    vec![ssf::ir::Definition::new(
                        "f",
                        vec![ssf::ir::Argument::new("x", ssf::types::Primitive::Float64)],
                        ssf::ir::Operation::new(*operator, 42.0, 42.0),
                        ssf::types::Primitive::Float64,
                    )],
                )
                .unwrap(),
                &COMPILE_CONFIGURATION,
            )
            .unwrap();
        }
    }

    #[test]
    fn compile_integer_comparison_operators() {
        for operator in &NUMBER_COMPARISON_OPERATORS {
            compile(
                &ssf::ir::Module::new(
                    vec![],
                    vec![ssf::ir::Definition::new(
                        "f",
                        vec![ssf::ir::Argument::new("x", ssf::types::Primitive::Float64)],
                        ssf::ir::Operation::new(*operator, 42, 42),
                        ssf::types::Primitive::Integer8,
                    )],
                )
                .unwrap(),
                &COMPILE_CONFIGURATION,
            )
            .unwrap();
        }
    }

    #[test]
    fn compile_float_comparison_operators() {
        for operator in &NUMBER_COMPARISON_OPERATORS {
            compile(
                &ssf::ir::Module::new(
                    vec![],
                    vec![ssf::ir::Definition::new(
                        "f",
                        vec![ssf::ir::Argument::new("x", ssf::types::Primitive::Float64)],
                        ssf::ir::Operation::new(*operator, 42.0, 42.0),
                        ssf::types::Primitive::Integer8,
                    )],
                )
                .unwrap(),
                &COMPILE_CONFIGURATION,
            )
            .unwrap();
        }
    }

    #[test]
    fn compile_nested_primitive_cases() {
        compile(
            &ssf::ir::Module::new(
                vec![],
                vec![ssf::ir::Definition::new(
                    "f",
                    vec![ssf::ir::Argument::new("x", ssf::types::Primitive::Float64)],
                    ssf::ir::PrimitiveCase::new(
                        ssf::types::Primitive::Float64,
                        ssf::ir::Variable::new("x"),
                        vec![ssf::ir::PrimitiveAlternative::new(
                            42.0,
                            ssf::ir::PrimitiveCase::new(
                                ssf::types::Primitive::Float64,
                                ssf::ir::Variable::new("x"),
                                vec![ssf::ir::PrimitiveAlternative::new(42.0, 42.0)],
                                None,
                            ),
                        )],
                        None,
                    ),
                    ssf::types::Primitive::Float64,
                )],
            )
            .unwrap(),
            &CompileConfiguration::new(None, Some("panic".into())),
        )
        .unwrap();
    }

    #[test]
    fn compile_nested_primitive_cases_with_default_alternatives() {
        compile(
            &ssf::ir::Module::new(
                vec![],
                vec![ssf::ir::Definition::new(
                    "f",
                    vec![ssf::ir::Argument::new("x", ssf::types::Primitive::Float64)],
                    ssf::ir::PrimitiveCase::new(
                        ssf::types::Primitive::Float64,
                        ssf::ir::Variable::new("x"),
                        vec![ssf::ir::PrimitiveAlternative::new(
                            42.0,
                            ssf::ir::PrimitiveCase::new(
                                ssf::types::Primitive::Float64,
                                ssf::ir::Variable::new("x"),
                                vec![ssf::ir::PrimitiveAlternative::new(42.0, 42.0)],
                                Some(ssf::ir::DefaultAlternative::new("x", 42.0)),
                            ),
                        )],
                        Some(ssf::ir::DefaultAlternative::new("x", 42.0)),
                    ),
                    ssf::types::Primitive::Float64,
                )],
            )
            .unwrap(),
            &CompileConfiguration::new(None, Some("panic".into())),
        )
        .unwrap();
    }

    #[test]
    fn compile_thunk_of_global_variable() {
        compile(
            &ssf::ir::Module::new(
                vec![],
                vec![ssf::ir::Definition::new(
                    "f",
                    vec![ssf::ir::Argument::new("x", ssf::types::Primitive::Float64)],
                    42.0,
                    ssf::types::Primitive::Float64,
                )],
            )
            .unwrap(),
            &COMPILE_CONFIGURATION,
        )
        .unwrap();
    }

    #[test]
    fn compile_thunk_evaluation() {
        compile(
            &ssf::ir::Module::new(
                vec![],
                vec![
                    ssf::ir::Definition::thunk(
                        "f",
                        vec![ssf::ir::Argument::new("x", ssf::types::Primitive::Float64)],
                        42.0,
                        ssf::types::Primitive::Float64,
                    ),
                    ssf::ir::Definition::new(
                        "g",
                        vec![ssf::ir::Argument::new("x", ssf::types::Primitive::Float64)],
                        ssf::ir::FunctionApplication::new(ssf::ir::Variable::new("f"), 42.0),
                        ssf::types::Primitive::Float64,
                    ),
                ],
            )
            .unwrap(),
            &COMPILE_CONFIGURATION,
        )
        .unwrap();
    }

    #[test]
    fn compile_thunk_evaluation_with_argument() {
        compile(
            &ssf::ir::Module::new(
                vec![],
                vec![
                    ssf::ir::Definition::thunk(
                        "f",
                        vec![ssf::ir::Argument::new("x", ssf::types::Primitive::Float64)],
                        42.0,
                        ssf::types::Primitive::Float64,
                    ),
                    ssf::ir::Definition::new(
                        "g",
                        vec![ssf::ir::Argument::new("x", ssf::types::Primitive::Float64)],
                        ssf::ir::FunctionApplication::new(ssf::ir::Variable::new("f"), 42.0),
                        ssf::types::Primitive::Float64,
                    ),
                ],
            )
            .unwrap(),
            &COMPILE_CONFIGURATION,
        )
        .unwrap();
    }

    #[test]
    fn compile_thunk_in_let_recursive_expression() {
        compile(
            &ssf::ir::Module::new(
                vec![],
                vec![ssf::ir::Definition::new(
                    "f",
                    vec![ssf::ir::Argument::new("x", ssf::types::Primitive::Float64)],
                    ssf::ir::LetRecursive::new(
                        vec![ssf::ir::Definition::thunk(
                            "g",
                            vec![ssf::ir::Argument::new("x", ssf::types::Primitive::Float64)],
                            ssf::ir::Variable::new("x"),
                            ssf::types::Primitive::Float64,
                        )],
                        ssf::ir::FunctionApplication::new(ssf::ir::Variable::new("g"), 42.0),
                    ),
                    ssf::types::Primitive::Float64,
                )],
            )
            .unwrap(),
            &COMPILE_CONFIGURATION,
        )
        .unwrap();
    }

    #[test]
    fn compile_let_recursive_expression_with_free_variable() {
        compile(
            &ssf::ir::Module::new(
                vec![],
                vec![ssf::ir::Definition::new(
                    "f",
                    vec![ssf::ir::Argument::new("x", ssf::types::Primitive::Float64)],
                    ssf::ir::LetRecursive::new(
                        vec![ssf::ir::Definition::new(
                            "g",
                            vec![ssf::ir::Argument::new("y", ssf::types::Primitive::Float64)],
                            ssf::ir::Variable::new("x"),
                            ssf::types::Primitive::Float64,
                        )],
                        42.0,
                    ),
                    ssf::types::Primitive::Float64,
                )],
            )
            .unwrap(),
            &COMPILE_CONFIGURATION,
        )
        .unwrap();
    }

    #[test]
    fn compile_let_recursive_expression_with_with_two_free_variables() {
        compile(
            &ssf::ir::Module::new(
                vec![],
                vec![ssf::ir::Definition::new(
                    "f",
                    vec![
                        ssf::ir::Argument::new("x", ssf::types::Primitive::Float64),
                        ssf::ir::Argument::new("y", ssf::types::Primitive::Float64),
                    ],
                    ssf::ir::LetRecursive::new(
                        vec![ssf::ir::Definition::new(
                            "g",
                            vec![ssf::ir::Argument::new("z", ssf::types::Primitive::Float64)],
                            ssf::ir::Operation::new(
                                ssf::ir::Operator::Add,
                                ssf::ir::Variable::new("x"),
                                ssf::ir::Variable::new("y"),
                            ),
                            ssf::types::Primitive::Float64,
                        )],
                        42.0,
                    ),
                    ssf::types::Primitive::Float64,
                )],
            )
            .unwrap(),
            &COMPILE_CONFIGURATION,
        )
        .unwrap();
    }

    #[test]
    fn compile_global_variable_reference_in_nested_let_recursive_expressions() {
        compile(
            &ssf::ir::Module::new(
                vec![],
                vec![ssf::ir::Definition::new(
                    "f",
                    vec![ssf::ir::Argument::new("x", ssf::types::Primitive::Float64)],
                    ssf::ir::LetRecursive::new(
                        vec![ssf::ir::Definition::new(
                            "g",
                            vec![ssf::ir::Argument::new("y", ssf::types::Primitive::Float64)],
                            ssf::ir::LetRecursive::new(
                                vec![ssf::ir::Definition::new(
                                    "g",
                                    vec![ssf::ir::Argument::new(
                                        "y",
                                        ssf::types::Primitive::Float64,
                                    )],
                                    ssf::ir::FunctionApplication::new(
                                        ssf::ir::Variable::new("f"),
                                        42.0,
                                    ),
                                    ssf::types::Primitive::Float64,
                                )],
                                42.0,
                            ),
                            ssf::types::Primitive::Float64,
                        )],
                        42.0,
                    ),
                    ssf::types::Primitive::Float64,
                )],
            )
            .unwrap(),
            &COMPILE_CONFIGURATION,
        )
        .unwrap();
    }

    #[test]
    fn compile_partial_application() {
        compile(
            &ssf::ir::Module::new(
                vec![],
                vec![
                    ssf::ir::Definition::new(
                        "f",
                        vec![
                            ssf::ir::Argument::new("x", ssf::types::Primitive::Float64),
                            ssf::ir::Argument::new("y", ssf::types::Primitive::Float64),
                        ],
                        42.0,
                        ssf::types::Primitive::Float64,
                    ),
                    ssf::ir::Definition::new(
                        "g",
                        vec![ssf::ir::Argument::new("x", ssf::types::Primitive::Float64)],
                        ssf::ir::FunctionApplication::new(ssf::ir::Variable::new("f"), 42.0),
                        ssf::types::Function::new(
                            ssf::types::Primitive::Float64,
                            ssf::types::Primitive::Float64,
                        ),
                    ),
                ],
            )
            .unwrap(),
            &COMPILE_CONFIGURATION,
        )
        .unwrap();
    }
}
