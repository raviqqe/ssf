mod closure_operation_compiler;
mod compile_configuration;
mod error;
mod expression_compiler;
mod expression_compiler_factory;
mod foreign_declaration_compiler;
mod function_application_compiler;
mod function_compiler;
mod function_compiler_factory;
mod global_variable;
mod instruction_compiler;
mod malloc_compiler;
mod module_compiler;
mod type_compiler;
mod utilities;

use closure_operation_compiler::ClosureOperationCompiler;
pub use compile_configuration::CompileConfiguration;
pub use error::CompileError;
use expression_compiler_factory::ExpressionCompilerFactory;
use foreign_declaration_compiler::ForeignDeclarationCompiler;
use function_application_compiler::FunctionApplicationCompiler;
use function_compiler_factory::FunctionCompilerFactory;
use malloc_compiler::MallocCompiler;
use module_compiler::ModuleCompiler;
use std::sync::Arc;
use type_compiler::TypeCompiler;

pub fn compile(
    ir_module: &ssf::ir::Module,
    compile_configuration: Arc<CompileConfiguration>,
) -> Result<Vec<u8>, CompileError> {
    let context = inkwell::context::Context::create();
    let module = Arc::new(context.create_module("main"));
    let type_compiler = TypeCompiler::new(&context);
    let closure_operation_compiler = ClosureOperationCompiler::new(&context, type_compiler.clone());
    let malloc_compiler = MallocCompiler::new(module.clone(), compile_configuration.clone());
    let function_application_compiler = FunctionApplicationCompiler::new(
        &context,
        module.clone(),
        type_compiler.clone(),
        closure_operation_compiler.clone(),
        malloc_compiler.clone(),
    );
    let expression_compiler_factory = ExpressionCompilerFactory::new(
        &context,
        function_application_compiler.clone(),
        type_compiler.clone(),
        closure_operation_compiler.clone(),
        malloc_compiler.clone(),
    );
    let function_compiler_factory = FunctionCompilerFactory::new(
        &context,
        module.clone(),
        expression_compiler_factory,
        type_compiler.clone(),
    );
    let foreign_declaration_compiler =
        ForeignDeclarationCompiler::new(&context, module.clone(), type_compiler.clone());

    ModuleCompiler::new(
        &context,
        module.clone(),
        function_compiler_factory,
        foreign_declaration_compiler,
        type_compiler,
        compile_configuration,
    )
    .compile(ir_module)?;

    Ok(module.write_bitcode_to_memory().as_slice().to_vec())
}

#[cfg(test)]
mod tests {
    use super::compile_configuration::COMPILE_CONFIGURATION;
    use super::*;

    const NUMBER_ARITHMETIC_OPERATORS: [ssf::ir::PrimitiveOperator; 4] = [
        ssf::ir::PrimitiveOperator::Add,
        ssf::ir::PrimitiveOperator::Subtract,
        ssf::ir::PrimitiveOperator::Multiply,
        ssf::ir::PrimitiveOperator::Divide,
    ];

    const NUMBER_COMPARISON_OPERATORS: [ssf::ir::PrimitiveOperator; 6] = [
        ssf::ir::PrimitiveOperator::Equal,
        ssf::ir::PrimitiveOperator::NotEqual,
        ssf::ir::PrimitiveOperator::GreaterThan,
        ssf::ir::PrimitiveOperator::GreaterThanOrEqual,
        ssf::ir::PrimitiveOperator::LessThan,
        ssf::ir::PrimitiveOperator::LessThanOrEqual,
    ];

    #[test]
    fn compile_() {
        compile(
            &ssf::ir::Module::new(vec![], vec![], vec![]).unwrap(),
            COMPILE_CONFIGURATION.clone(),
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
            COMPILE_CONFIGURATION.clone(),
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
            COMPILE_CONFIGURATION.clone(),
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
                vec![],
                vec![ssf::ir::Definition::new(
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
                )],
            )
            .unwrap(),
            COMPILE_CONFIGURATION.clone(),
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
            COMPILE_CONFIGURATION.clone(),
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
                vec![],
                vec![ssf::ir::Definition::new(
                    "f",
                    vec![ssf::ir::Argument::new("x", algebraic_type.clone())],
                    ssf::ir::AlgebraicCase::new(
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
            COMPILE_CONFIGURATION.clone(),
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
                vec![],
                vec![ssf::ir::Definition::new(
                    "f",
                    vec![ssf::ir::Argument::new("x", algebraic_type.clone())],
                    ssf::ir::AlgebraicCase::new(
                        ssf::ir::Variable::new("x"),
                        vec![ssf::ir::AlgebraicAlternative::new(
                            ssf::ir::Constructor::new(algebraic_type.clone(), 0),
                            vec![],
                            ssf::ir::AlgebraicCase::new(
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
            COMPILE_CONFIGURATION.clone(),
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
                vec![],
                vec![ssf::ir::Definition::new(
                    "f",
                    vec![ssf::ir::Argument::new("x", algebraic_type.clone())],
                    ssf::ir::AlgebraicCase::new(
                        ssf::ir::Variable::new("x"),
                        vec![ssf::ir::AlgebraicAlternative::new(
                            ssf::ir::Constructor::new(algebraic_type.clone(), 0),
                            vec![],
                            ssf::ir::AlgebraicCase::new(
                                ssf::ir::Variable::new("x"),
                                vec![ssf::ir::AlgebraicAlternative::new(
                                    ssf::ir::Constructor::new(algebraic_type, 0),
                                    vec![],
                                    42.0,
                                )],
                                Some(42.0.into()),
                            ),
                        )],
                        Some(42.0.into()),
                    ),
                    ssf::types::Primitive::Float64,
                )],
            )
            .unwrap(),
            COMPILE_CONFIGURATION.clone(),
        )
        .unwrap();
    }

    #[test]
    fn compile_bitcast() {
        compile(
            &ssf::ir::Module::new(
                vec![],
                vec![],
                vec![ssf::ir::Definition::new(
                    "f",
                    vec![ssf::ir::Argument::new("x", ssf::types::Primitive::Float64)],
                    ssf::ir::Bitcast::new(42, ssf::types::Primitive::Float64),
                    ssf::types::Primitive::Float64,
                )],
            )
            .unwrap(),
            COMPILE_CONFIGURATION.clone(),
        )
        .unwrap();
    }

    #[test]
    fn compile_bitcast_of_algebraic_data_types() {
        let algebraic_type_1 =
            ssf::types::Algebraic::new(vec![ssf::types::Constructor::unboxed(vec![
                ssf::types::Primitive::Integer64.into(),
            ])]);
        let algebraic_type_2 =
            ssf::types::Algebraic::new(vec![ssf::types::Constructor::unboxed(vec![
                ssf::types::Primitive::Float64.into(),
            ])]);

        compile(
            &ssf::ir::Module::new(
                vec![],
                vec![],
                vec![ssf::ir::Definition::new(
                    "f",
                    vec![ssf::ir::Argument::new("x", algebraic_type_1)],
                    ssf::ir::Bitcast::new(ssf::ir::Variable::new("x"), algebraic_type_2.clone()),
                    algebraic_type_2,
                )],
            )
            .unwrap(),
            COMPILE_CONFIGURATION.clone(),
        )
        .unwrap();
    }

    #[test]
    fn compile_bitcast_from_integer_to_algebraic_data_type() {
        let algebraic_type =
            ssf::types::Algebraic::new(vec![ssf::types::Constructor::unboxed(vec![
                ssf::types::Primitive::Integer64.into(),
            ])]);

        compile(
            &ssf::ir::Module::new(
                vec![],
                vec![],
                vec![ssf::ir::Definition::new(
                    "f",
                    vec![ssf::ir::Argument::new(
                        "x",
                        ssf::types::Primitive::Integer64,
                    )],
                    ssf::ir::Bitcast::new(ssf::ir::Variable::new("x"), algebraic_type.clone()),
                    algebraic_type,
                )],
            )
            .unwrap(),
            COMPILE_CONFIGURATION.clone(),
        )
        .unwrap();
    }

    #[test]
    fn fail_to_compile_bitcast_of_algebraic_data_types() {
        let algebraic_type_1 =
            ssf::types::Algebraic::new(vec![ssf::types::Constructor::unboxed(vec![])]);
        let algebraic_type_2 =
            ssf::types::Algebraic::new(vec![ssf::types::Constructor::unboxed(vec![
                ssf::types::Primitive::Float64.into(),
            ])]);

        assert!(matches!(
            compile(
                &ssf::ir::Module::new(
                    vec![],
                    vec![],
                    vec![ssf::ir::Definition::new(
                        "f",
                        vec![ssf::ir::Argument::new("x", algebraic_type_1)],
                        ssf::ir::Bitcast::new(
                            ssf::ir::Variable::new("x"),
                            algebraic_type_2.clone()
                        ),
                        algebraic_type_2,
                    )],
                )
                .unwrap(),
                COMPILE_CONFIGURATION.clone(),
            ),
            Err(_)
        ));
    }

    #[test]
    fn compile_function_applications() {
        compile(
            &ssf::ir::Module::new(
                vec![],
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
            COMPILE_CONFIGURATION.clone(),
        )
        .unwrap();
    }

    #[test]
    fn compile_integer_arithmetic_operators() {
        for operator in &NUMBER_ARITHMETIC_OPERATORS {
            compile(
                &ssf::ir::Module::new(
                    vec![],
                    vec![],
                    vec![ssf::ir::Definition::new(
                        "f",
                        vec![ssf::ir::Argument::new("x", ssf::types::Primitive::Float64)],
                        ssf::ir::PrimitiveOperation::new(*operator, 42, 42),
                        ssf::types::Primitive::Integer64,
                    )],
                )
                .unwrap(),
                COMPILE_CONFIGURATION.clone(),
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
                    vec![],
                    vec![ssf::ir::Definition::new(
                        "f",
                        vec![ssf::ir::Argument::new("x", ssf::types::Primitive::Float64)],
                        ssf::ir::PrimitiveOperation::new(*operator, 42.0, 42.0),
                        ssf::types::Primitive::Float64,
                    )],
                )
                .unwrap(),
                COMPILE_CONFIGURATION.clone(),
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
                    vec![],
                    vec![ssf::ir::Definition::new(
                        "f",
                        vec![ssf::ir::Argument::new("x", ssf::types::Primitive::Float64)],
                        ssf::ir::PrimitiveOperation::new(*operator, 42, 42),
                        ssf::types::Primitive::Integer8,
                    )],
                )
                .unwrap(),
                COMPILE_CONFIGURATION.clone(),
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
                    vec![],
                    vec![ssf::ir::Definition::new(
                        "f",
                        vec![ssf::ir::Argument::new("x", ssf::types::Primitive::Float64)],
                        ssf::ir::PrimitiveOperation::new(*operator, 42.0, 42.0),
                        ssf::types::Primitive::Integer8,
                    )],
                )
                .unwrap(),
                COMPILE_CONFIGURATION.clone(),
            )
            .unwrap();
        }
    }

    #[test]
    fn compile_nested_primitive_cases() {
        compile(
            &ssf::ir::Module::new(
                vec![],
                vec![],
                vec![ssf::ir::Definition::new(
                    "f",
                    vec![ssf::ir::Argument::new("x", ssf::types::Primitive::Float64)],
                    ssf::ir::PrimitiveCase::new(
                        ssf::ir::Variable::new("x"),
                        vec![ssf::ir::PrimitiveAlternative::new(
                            42.0,
                            ssf::ir::PrimitiveCase::new(
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
            COMPILE_CONFIGURATION.clone(),
        )
        .unwrap();
    }

    #[test]
    fn compile_nested_primitive_cases_with_default_alternatives() {
        compile(
            &ssf::ir::Module::new(
                vec![],
                vec![],
                vec![ssf::ir::Definition::new(
                    "f",
                    vec![ssf::ir::Argument::new("x", ssf::types::Primitive::Float64)],
                    ssf::ir::PrimitiveCase::new(
                        ssf::ir::Variable::new("x"),
                        vec![ssf::ir::PrimitiveAlternative::new(
                            42.0,
                            ssf::ir::PrimitiveCase::new(
                                ssf::ir::Variable::new("x"),
                                vec![ssf::ir::PrimitiveAlternative::new(42.0, 42.0)],
                                Some(42.0.into()),
                            ),
                        )],
                        Some(42.0.into()),
                    ),
                    ssf::types::Primitive::Float64,
                )],
            )
            .unwrap(),
            COMPILE_CONFIGURATION.clone(),
        )
        .unwrap();
    }

    #[test]
    fn compile_thunk_of_global_variable() {
        compile(
            &ssf::ir::Module::new(
                vec![],
                vec![],
                vec![ssf::ir::Definition::new(
                    "f",
                    vec![ssf::ir::Argument::new("x", ssf::types::Primitive::Float64)],
                    42.0,
                    ssf::types::Primitive::Float64,
                )],
            )
            .unwrap(),
            COMPILE_CONFIGURATION.clone(),
        )
        .unwrap();
    }

    #[test]
    fn compile_thunk_evaluation() {
        compile(
            &ssf::ir::Module::new(
                vec![],
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
            COMPILE_CONFIGURATION.clone(),
        )
        .unwrap();
    }

    #[test]
    fn compile_thunk_evaluation_with_argument() {
        compile(
            &ssf::ir::Module::new(
                vec![],
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
            COMPILE_CONFIGURATION.clone(),
        )
        .unwrap();
    }

    #[test]
    fn compile_thunk_in_let_recursive_expression() {
        compile(
            &ssf::ir::Module::new(
                vec![],
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
            COMPILE_CONFIGURATION.clone(),
        )
        .unwrap();
    }

    #[test]
    fn compile_let_recursive_expression_with_free_variable() {
        compile(
            &ssf::ir::Module::new(
                vec![],
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
            COMPILE_CONFIGURATION.clone(),
        )
        .unwrap();
    }

    #[test]
    fn compile_let_recursive_expression_with_with_two_free_variables() {
        compile(
            &ssf::ir::Module::new(
                vec![],
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
                            ssf::ir::PrimitiveOperation::new(
                                ssf::ir::PrimitiveOperator::Add,
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
            COMPILE_CONFIGURATION.clone(),
        )
        .unwrap();
    }

    #[test]
    fn compile_global_variable_reference_in_nested_let_recursive_expressions() {
        compile(
            &ssf::ir::Module::new(
                vec![],
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
            COMPILE_CONFIGURATION.clone(),
        )
        .unwrap();
    }

    mod partial_application {
        use super::*;

        #[test]
        fn compile_with_1_argument_and_2_arity() {
            compile(
                &ssf::ir::Module::new(
                    vec![],
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
                COMPILE_CONFIGURATION.clone(),
            )
            .unwrap();
        }

        #[test]
        fn compile_with_1_argument_and_3_arity() {
            compile(
                &ssf::ir::Module::new(
                    vec![],
                    vec![],
                    vec![
                        ssf::ir::Definition::new(
                            "f",
                            vec![
                                ssf::ir::Argument::new("x", ssf::types::Primitive::Float64),
                                ssf::ir::Argument::new("y", ssf::types::Primitive::Float64),
                                ssf::ir::Argument::new("z", ssf::types::Primitive::Float64),
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
                                ssf::types::Function::new(
                                    ssf::types::Primitive::Float64,
                                    ssf::types::Primitive::Float64,
                                ),
                            ),
                        ),
                    ],
                )
                .unwrap(),
                COMPILE_CONFIGURATION.clone(),
            )
            .unwrap();
        }

        #[test]
        fn compile_with_2_argument_and_3_arity() {
            compile(
                &ssf::ir::Module::new(
                    vec![],
                    vec![],
                    vec![
                        ssf::ir::Definition::new(
                            "f",
                            vec![
                                ssf::ir::Argument::new("x", ssf::types::Primitive::Float64),
                                ssf::ir::Argument::new("y", ssf::types::Primitive::Float64),
                                ssf::ir::Argument::new("z", ssf::types::Primitive::Float64),
                            ],
                            42.0,
                            ssf::types::Primitive::Float64,
                        ),
                        ssf::ir::Definition::new(
                            "g",
                            vec![ssf::ir::Argument::new("x", ssf::types::Primitive::Float64)],
                            ssf::ir::FunctionApplication::new(
                                ssf::ir::FunctionApplication::new(
                                    ssf::ir::Variable::new("f"),
                                    42.0,
                                ),
                                42.0,
                            ),
                            ssf::types::Function::new(
                                ssf::types::Primitive::Float64,
                                ssf::types::Primitive::Float64,
                            ),
                        ),
                    ],
                )
                .unwrap(),
                COMPILE_CONFIGURATION.clone(),
            )
            .unwrap();
        }
    }

    mod foreign_declarations {
        use super::*;

        #[test]
        fn compile_foreign_declaration() {
            compile(
                &ssf::ir::Module::new(
                    vec![ssf::ir::ForeignDeclaration::new(
                        "f",
                        "g",
                        ssf::types::Function::new(
                            ssf::types::Primitive::Float64,
                            ssf::types::Primitive::Float64,
                        ),
                    )],
                    vec![],
                    vec![ssf::ir::Definition::new(
                        "h",
                        vec![ssf::ir::Argument::new("x", ssf::types::Primitive::Float64)],
                        ssf::ir::FunctionApplication::new(
                            ssf::ir::Variable::new("f"),
                            ssf::ir::Variable::new("x"),
                        ),
                        ssf::types::Primitive::Float64,
                    )],
                )
                .unwrap(),
                COMPILE_CONFIGURATION.clone(),
            )
            .unwrap();
        }

        #[test]
        fn compile_foreign_declaration_with_multiple_arguments() {
            compile(
                &ssf::ir::Module::new(
                    vec![ssf::ir::ForeignDeclaration::new(
                        "f",
                        "g",
                        ssf::types::Function::new(
                            ssf::types::Primitive::Float64,
                            ssf::types::Function::new(
                                ssf::types::Primitive::Float64,
                                ssf::types::Primitive::Float64,
                            ),
                        ),
                    )],
                    vec![],
                    vec![ssf::ir::Definition::new(
                        "h",
                        vec![ssf::ir::Argument::new("x", ssf::types::Primitive::Float64)],
                        ssf::ir::FunctionApplication::new(
                            ssf::ir::FunctionApplication::new(
                                ssf::ir::Variable::new("f"),
                                ssf::ir::Variable::new("x"),
                            ),
                            ssf::ir::Variable::new("x"),
                        ),
                        ssf::types::Primitive::Float64,
                    )],
                )
                .unwrap(),
                COMPILE_CONFIGURATION.clone(),
            )
            .unwrap();
        }
    }
}
