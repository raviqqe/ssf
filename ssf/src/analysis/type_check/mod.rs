mod error;
mod type_checker;

use crate::ir::*;
pub use error::*;
use type_checker::*;

pub fn check_types(module: &Module) -> Result<(), TypeCheckError> {
    TypeChecker::new().check(module)
}

#[cfg(test)]
mod tests {
    use super::check_types;
    use super::error::*;
    use crate::ir::*;
    use crate::types::{self, Type};

    #[test]
    fn check_types_with_empty_modules() {
        assert_eq!(check_types(&Module::new(vec![], vec![], vec![])), Ok(()));
    }

    #[test]
    fn check_types_of_variables() {
        let module = Module::new(
            vec![],
            vec![],
            vec![Definition::new(
                "f",
                vec![Argument::new("x", types::Primitive::Float64)],
                Variable::new("x"),
                types::Primitive::Float64,
            )],
        );
        assert_eq!(check_types(&module), Ok(()));
    }

    #[test]
    fn fail_to_check_types_of_variables() {
        let module = Module::new(
            vec![],
            vec![],
            vec![
                Definition::new(
                    "f",
                    vec![Argument::new("x", types::Primitive::Float64)],
                    42.0,
                    types::Primitive::Float64,
                ),
                Definition::new(
                    "g",
                    vec![Argument::new("x", types::Primitive::Float64)],
                    Variable::new("f"),
                    types::Primitive::Float64,
                ),
            ],
        );

        assert!(matches!(
            check_types(&module),
            Err(TypeCheckError::TypesNotMatched(_, _))
        ));
    }

    #[test]
    fn check_types_of_functions() {
        let module = Module::new(
            vec![],
            vec![],
            vec![Definition::new(
                "f",
                vec![Argument::new("x", types::Primitive::Float64)],
                42.0,
                types::Primitive::Float64,
            )],
        );

        assert_eq!(check_types(&module), Ok(()));
    }

    #[test]
    fn fail_to_check_types_of_functions() {
        let module = Module::new(
            vec![],
            vec![],
            vec![
                Definition::new(
                    "f",
                    vec![Argument::new("x", types::Primitive::Float64)],
                    42.0,
                    types::Primitive::Float64,
                ),
                Definition::new(
                    "g",
                    vec![Argument::new("x", types::Primitive::Float64)],
                    Variable::new("f"),
                    types::Primitive::Float64,
                ),
            ],
        );

        assert!(matches!(
            check_types(&module),
            Err(TypeCheckError::TypesNotMatched(_, _))
        ));
    }

    #[test]
    fn check_types_of_function_applications() {
        let module = Module::new(
            vec![],
            vec![],
            vec![
                Definition::new(
                    "f",
                    vec![Argument::new("x", types::Primitive::Float64)],
                    42.0,
                    types::Primitive::Float64,
                ),
                Definition::new(
                    "g",
                    vec![Argument::new("x", types::Primitive::Float64)],
                    FunctionApplication::new(Variable::new("f"), 42.0),
                    types::Primitive::Float64,
                ),
            ],
        );

        assert_eq!(check_types(&module), Ok(()));
    }

    #[test]
    fn fail_to_check_types_of_function_applications() {
        let module = Module::new(
            vec![],
            vec![],
            vec![
                Definition::new(
                    "f",
                    vec![Argument::new("x", types::Primitive::Float64)],
                    42.0,
                    types::Primitive::Float64,
                ),
                Definition::new(
                    "g",
                    vec![Argument::new("x", types::Primitive::Float64)],
                    FunctionApplication::new(
                        FunctionApplication::new(Variable::new("f"), 42.0),
                        42.0,
                    ),
                    types::Primitive::Float64,
                ),
            ],
        );

        assert!(matches!(
            check_types(&module),
            Err(TypeCheckError::FunctionExpected(_))
        ));
    }

    #[test]
    fn fail_to_check_types_because_of_missing_variables() {
        let module = Module::new(
            vec![],
            vec![],
            vec![Definition::new(
                "f",
                vec![Argument::new("x", types::Primitive::Float64)],
                Variable::new("y"),
                types::Primitive::Float64,
            )],
        );

        assert!(matches!(
            check_types(&module),
            Err(TypeCheckError::VariableNotFound(_))
        ));
    }

    #[test]
    fn check_types_of_nested_let_expressions() {
        let module = Module::new(
            vec![],
            vec![],
            vec![Definition::new(
                "f",
                vec![Argument::new("x", types::Primitive::Float64)],
                Let::new(
                    "y",
                    types::Primitive::Float64,
                    42.0,
                    Let::new(
                        "z",
                        types::Primitive::Float64,
                        Variable::new("y"),
                        Variable::new("z"),
                    ),
                ),
                types::Primitive::Float64,
            )],
        );

        assert_eq!(check_types(&module), Ok(()));
    }

    #[test]
    fn fail_to_check_types_of_let_expression() {
        let module = Module::new(
            vec![],
            vec![],
            vec![
                Definition::new(
                    "f",
                    vec![Argument::new("x", types::Primitive::Float64)],
                    42.0,
                    types::Primitive::Float64,
                ),
                Definition::new(
                    "g",
                    vec![Argument::new("x", types::Primitive::Float64)],
                    Let::new(
                        "y",
                        types::Primitive::Float64,
                        Variable::new("f"),
                        Variable::new("y"),
                    ),
                    types::Primitive::Float64,
                ),
            ],
        );

        assert!(matches!(
            check_types(&module),
            Err(TypeCheckError::TypesNotMatched(_, _))
        ));
    }

    #[test]
    fn check_types_of_declarations() {
        let module = Module::new(
            vec![],
            vec![Declaration::new(
                "f",
                types::Function::new(types::Primitive::Float64, types::Primitive::Float64),
            )],
            vec![Definition::new(
                "g",
                vec![Argument::new("x", types::Primitive::Float64)],
                FunctionApplication::new(Variable::new("f"), Variable::new("x")),
                types::Primitive::Float64,
            )],
        );
        assert_eq!(check_types(&module), Ok(()));
    }

    #[test]
    fn fail_to_check_types_of_declarations() {
        let module = Module::new(
            vec![],
            vec![Declaration::new(
                "f",
                types::Function::new(types::Primitive::Float64, types::Primitive::Float64),
            )],
            vec![Definition::new(
                "g",
                vec![Argument::new("x", types::Primitive::Float64)],
                Variable::new("f"),
                types::Primitive::Float64,
            )],
        );

        assert!(matches!(
            check_types(&module),
            Err(TypeCheckError::TypesNotMatched(_, _))
        ));
    }

    mod case_expressions {
        use super::*;

        mod algebraic {
            use super::*;

            #[test]
            fn check_case_expressions_only_with_default_alternative() {
                let algebraic_type = types::Algebraic::new(vec![types::Constructor::boxed(vec![])]);

                assert_eq!(
                    check_types(&Module::new(
                        vec![],
                        vec![],
                        vec![Definition::new(
                            "f",
                            vec![Argument::new("x", algebraic_type,)],
                            AlgebraicCase::new(Variable::new("x"), vec![], Some(42.0.into()),),
                            types::Primitive::Float64,
                        )]
                    )),
                    Ok(())
                );
            }

            #[test]
            fn check_case_expressions_with_one_alternative() {
                let algebraic_type = types::Algebraic::new(vec![types::Constructor::boxed(vec![])]);

                assert_eq!(
                    check_types(&Module::new(
                        vec![],
                        vec![],
                        vec![Definition::new(
                            "f",
                            vec![Argument::new("x", algebraic_type.clone())],
                            AlgebraicCase::new(
                                Variable::new("x"),
                                vec![AlgebraicAlternative::new(
                                    Constructor::new(algebraic_type, 0),
                                    vec![],
                                    42.0
                                )],
                                None
                            ),
                            types::Primitive::Float64,
                        )]
                    )),
                    Ok(())
                );
            }

            #[test]
            fn check_case_expressions_with_deconstruction() {
                let algebraic_type = types::Algebraic::new(vec![types::Constructor::boxed(vec![
                    types::Primitive::Float64.into(),
                ])]);

                assert_eq!(
                    check_types(&Module::new(
                        vec![],
                        vec![],
                        vec![Definition::new(
                            "f",
                            vec![Argument::new("x", algebraic_type.clone())],
                            AlgebraicCase::new(
                                Variable::new("x"),
                                vec![AlgebraicAlternative::new(
                                    Constructor::new(algebraic_type, 0),
                                    vec!["y".into()],
                                    Variable::new("y")
                                )],
                                None
                            ),
                            types::Primitive::Float64,
                        )]
                    )),
                    Ok(())
                );
            }

            #[test]
            fn fail_to_check_case_expressions_without_alternatives() {
                let algebraic_type = types::Algebraic::new(vec![types::Constructor::boxed(vec![])]);

                let module = Module::new(
                    vec![],
                    vec![],
                    vec![Definition::new(
                        "f",
                        vec![Argument::new("x", algebraic_type)],
                        AlgebraicCase::new(Variable::new("x"), vec![], None),
                        types::Primitive::Float64,
                    )],
                );

                assert!(matches!(
                    check_types(&module),
                    Err(TypeCheckError::NoAlternativeFound(_))
                ));
            }

            #[test]
            fn fail_to_check_case_expressions_with_inconsistent_alternative_types() {
                let algebraic_type = types::Algebraic::new(vec![types::Constructor::boxed(vec![])]);
                let module = Module::new(
                    vec![],
                    vec![],
                    vec![Definition::with_environment(
                        "f",
                        vec![],
                        vec![Argument::new(
                            "x",
                            types::Algebraic::new(vec![types::Constructor::boxed(vec![])]),
                        )],
                        AlgebraicCase::new(
                            Variable::new("x"),
                            vec![
                                AlgebraicAlternative::new(
                                    Constructor::new(algebraic_type.clone(), 0),
                                    vec![],
                                    Variable::new("x"),
                                ),
                                AlgebraicAlternative::new(
                                    Constructor::new(algebraic_type, 0),
                                    vec![],
                                    42.0,
                                ),
                            ],
                            None,
                        ),
                        types::Primitive::Float64,
                    )],
                );

                assert!(matches!(
                    check_types(&module),
                    Err(TypeCheckError::TypesNotMatched(_, _))
                ));
            }

            #[test]
            fn check_case_expressions_with_recursive_algebraic_types() {
                let algebraic_type =
                    types::Algebraic::new(vec![types::Constructor::boxed(vec![Type::Index(0)])]);

                assert_eq!(
                    check_types(&Module::new(
                        vec![],
                        vec![],
                        vec![Definition::with_environment(
                            "f",
                            vec![],
                            vec![Argument::new("x", algebraic_type.clone())],
                            AlgebraicCase::new(
                                Variable::new("x"),
                                vec![AlgebraicAlternative::new(
                                    Constructor::new(algebraic_type.clone(), 0),
                                    vec!["y".into()],
                                    Variable::new("y"),
                                )],
                                None,
                            ),
                            algebraic_type,
                        )]
                    )),
                    Ok(())
                );
            }

            #[test]
            fn fail_for_unmatched_case_type() {
                let algebraic_type = types::Algebraic::new(vec![types::Constructor::boxed(vec![])]);
                let other_algebraic_type =
                    types::Algebraic::new(vec![types::Constructor::unboxed(vec![])]);

                assert!(matches!(
                    check_types(&Module::new(
                        vec![],
                        vec![],
                        vec![Definition::with_environment(
                            "f",
                            vec![],
                            vec![Argument::new("x", algebraic_type)],
                            AlgebraicCase::new(
                                Variable::new("x"),
                                vec![AlgebraicAlternative::new(
                                    Constructor::new(other_algebraic_type, 0),
                                    vec![],
                                    42.0
                                )],
                                None
                            ),
                            types::Primitive::Float64,
                        )],
                    )),
                    Err(TypeCheckError::TypesNotMatched(_, _))
                ));
            }
        }

        mod primitive {
            use super::*;

            #[test]
            fn check_case_expressions_only_with_default_alternative() {
                assert_eq!(
                    check_types(&Module::new(
                        vec![],
                        vec![],
                        vec![Definition::with_environment(
                            "f",
                            vec![],
                            vec![Argument::new("x", types::Primitive::Float64)],
                            PrimitiveCase::new(42.0, vec![], Some(42.0.into()),),
                            types::Primitive::Float64,
                        )]
                    )),
                    Ok(())
                );
            }

            #[test]
            fn check_case_expressions_with_one_alternative() {
                assert_eq!(
                    check_types(&Module::new(
                        vec![],
                        vec![],
                        vec![Definition::with_environment(
                            "f",
                            vec![],
                            vec![Argument::new("x", types::Primitive::Float64)],
                            PrimitiveCase::new(
                                42.0,
                                vec![PrimitiveAlternative::new(42.0, 42.0)],
                                None
                            ),
                            types::Primitive::Float64,
                        )],
                    )),
                    Ok(())
                );
            }

            #[test]
            fn check_case_expressions_with_one_alternative_and_default_alternative() {
                assert_eq!(
                    check_types(&Module::new(
                        vec![],
                        vec![],
                        vec![Definition::with_environment(
                            "f",
                            vec![],
                            vec![Argument::new("x", types::Primitive::Float64)],
                            PrimitiveCase::new(
                                42.0,
                                vec![PrimitiveAlternative::new(42.0, 42.0)],
                                Some(42.0.into())
                            ),
                            types::Primitive::Float64,
                        )],
                    )),
                    Ok(())
                );
            }

            #[test]
            fn fail_for_unmatched_case_type() {
                assert!(matches!(
                    check_types(&Module::new(
                        vec![],
                        vec![],
                        vec![Definition::with_environment(
                            "f",
                            vec![],
                            vec![Argument::new("x", types::Primitive::Float64)],
                            PrimitiveCase::new(
                                42.0,
                                vec![PrimitiveAlternative::new(42, 42.0)],
                                Some(42.0.into())
                            ),
                            types::Primitive::Float64,
                        )],
                    )),
                    Err(TypeCheckError::TypesNotMatched(_, _))
                ));
            }
        }
    }

    mod constructor_applications {
        use super::*;

        #[test]
        fn check_constructor_applications_with_no_arguments() {
            let algebraic_type = types::Algebraic::new(vec![types::Constructor::boxed(vec![])]);

            assert_eq!(
                check_types(&Module::new(
                    vec![],
                    vec![],
                    vec![Definition::with_environment(
                        "f",
                        vec![],
                        vec![Argument::new("x", types::Primitive::Float64)],
                        ConstructorApplication::new(
                            Constructor::new(algebraic_type.clone(), 0),
                            vec![],
                        ),
                        algebraic_type,
                    )],
                )),
                Ok(())
            );
        }

        #[test]
        fn check_constructor_applications_with_arguments() {
            let algebraic_type = types::Algebraic::new(vec![types::Constructor::boxed(vec![
                types::Primitive::Float64.into(),
            ])]);

            assert_eq!(
                check_types(&Module::new(
                    vec![],
                    vec![],
                    vec![Definition::with_environment(
                        "f",
                        vec![],
                        vec![Argument::new("x", types::Primitive::Float64)],
                        ConstructorApplication::new(
                            Constructor::new(algebraic_type.clone(), 0),
                            vec![42.0.into()],
                        ),
                        algebraic_type,
                    )],
                )),
                Ok(())
            );
        }

        #[test]
        fn fail_to_check_constructor_applications_with_wrong_number_of_arguments() {
            let algebraic_type = types::Algebraic::new(vec![types::Constructor::boxed(vec![
                types::Primitive::Float64.into(),
            ])]);
            let module = Module::new(
                vec![],
                vec![],
                vec![Definition::with_environment(
                    "f",
                    vec![],
                    vec![Argument::new("x", types::Primitive::Float64)],
                    ConstructorApplication::new(
                        Constructor::new(algebraic_type.clone(), 0),
                        vec![42.0.into(), 42.0.into()],
                    ),
                    algebraic_type,
                )],
            );

            assert!(matches!(
                check_types(&module),
                Err(TypeCheckError::WrongArgumentsLength(_))
            ));
        }

        #[test]
        fn fail_to_check_constructor_applications_with_wrong_argument_type() {
            let algebraic_type = types::Algebraic::new(vec![types::Constructor::boxed(vec![
                types::Primitive::Float64.into(),
            ])]);
            let module = Module::new(
                vec![],
                vec![],
                vec![Definition::with_environment(
                    "f",
                    vec![],
                    vec![Argument::new("x", types::Primitive::Float64)],
                    ConstructorApplication::new(
                        Constructor::new(algebraic_type.clone(), 0),
                        vec![ConstructorApplication::new(
                            Constructor::new(algebraic_type.clone(), 0),
                            vec![42.0.into()],
                        )
                        .into()],
                    ),
                    algebraic_type,
                )],
            );

            assert!(matches!(
                check_types(&module),
                Err(TypeCheckError::TypesNotMatched(_, _))
            ));
        }

        #[test]
        fn check_constructor_applications_of_recursive_algebraic_types() {
            let algebraic_type =
                types::Algebraic::new(vec![types::Constructor::boxed(vec![Type::Index(0)])]);

            assert_eq!(
                check_types(&Module::new(
                    vec![],
                    vec![],
                    vec![Definition::new(
                        "f",
                        vec![Argument::new("x", algebraic_type.clone())],
                        ConstructorApplication::new(
                            Constructor::new(algebraic_type.clone(), 0),
                            vec![Variable::new("x").into()],
                        ),
                        algebraic_type,
                    )],
                )),
                Ok(())
            );
        }
    }

    #[test]
    fn check_bitcast() {
        let module = Module::new(
            vec![],
            vec![],
            vec![Definition::with_environment(
                "f",
                vec![],
                vec![Argument::new("x", types::Primitive::Float64)],
                Bitcast::new(42, types::Primitive::Float64),
                types::Primitive::Float64,
            )],
        );
        assert_eq!(check_types(&module), Ok(()));
    }

    #[test]
    fn check_equality_operator() {
        let module = Module::new(
            vec![],
            vec![],
            vec![Definition::with_environment(
                "f",
                vec![],
                vec![Argument::new("x", types::Primitive::Float64)],
                PrimitiveOperation::new(PrimitiveOperator::Equal, 42.0, 42.0),
                types::Primitive::Integer8,
            )],
        );
        assert_eq!(check_types(&module), Ok(()));
    }

    mod foreign_declarations {
        use super::*;

        #[test]
        fn check_types_of_foreign_declarations() {
            let module = Module::new(
                vec![ForeignDeclaration::new(
                    "f",
                    "g",
                    types::Function::new(types::Primitive::Float64, types::Primitive::Float64),
                )],
                vec![],
                vec![Definition::new(
                    "g",
                    vec![Argument::new("x", types::Primitive::Float64)],
                    FunctionApplication::new(Variable::new("f"), Variable::new("x")),
                    types::Primitive::Float64,
                )],
            );
            assert_eq!(check_types(&module), Ok(()));
        }

        #[test]
        fn fail_to_check_types_of_foreign_declarations() {
            let module = Module::new(
                vec![ForeignDeclaration::new(
                    "f",
                    "g",
                    types::Function::new(types::Primitive::Float64, types::Primitive::Float64),
                )],
                vec![],
                vec![Definition::new(
                    "g",
                    vec![Argument::new("x", types::Primitive::Float64)],
                    Variable::new("f"),
                    types::Primitive::Float64,
                )],
            );

            assert!(matches!(
                check_types(&module),
                Err(TypeCheckError::TypesNotMatched(_, _))
            ));
        }
    }
}
