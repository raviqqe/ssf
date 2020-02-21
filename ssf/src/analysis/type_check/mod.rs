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
    use crate::types;

    #[test]
    fn check_types_with_empty_modules() {
        assert_eq!(
            check_types(&Module::without_validation(vec![], vec![], vec![])),
            Ok(())
        );
    }

    #[test]
    fn check_types_of_variables() {
        let module = Module::without_validation(
            vec![],
            vec![ValueDefinition::new("x", 42.0, types::Primitive::Float64).into()],
            vec![],
        );
        assert_eq!(check_types(&module), Ok(()));
    }

    #[test]
    fn fail_to_check_types_of_variables() {
        let module = Module::without_validation(
            vec![],
            vec![
                FunctionDefinition::new(
                    "f",
                    vec![Argument::new("x", types::Primitive::Float64)],
                    42.0,
                    types::Primitive::Float64,
                )
                .into(),
                ValueDefinition::new("x", Variable::new("f"), types::Primitive::Float64).into(),
            ],
            vec![],
        );

        assert_eq!(check_types(&module), Err(TypeCheckError));
    }

    #[test]
    fn check_types_of_functions() {
        let module = Module::without_validation(
            vec![],
            vec![FunctionDefinition::new(
                "f",
                vec![Argument::new("x", types::Primitive::Float64)],
                42.0,
                types::Primitive::Float64,
            )
            .into()],
            vec![],
        );

        assert_eq!(check_types(&module), Ok(()));
    }

    #[test]
    fn fail_to_check_types_of_functions() {
        let module = Module::without_validation(
            vec![],
            vec![
                FunctionDefinition::new(
                    "f",
                    vec![Argument::new("x", types::Primitive::Float64)],
                    42.0,
                    types::Primitive::Float64,
                )
                .into(),
                FunctionDefinition::new(
                    "g",
                    vec![Argument::new("x", types::Primitive::Float64)],
                    Variable::new("f"),
                    types::Primitive::Float64,
                )
                .into(),
            ],
            vec![],
        );

        assert_eq!(check_types(&module), Err(TypeCheckError));
    }

    #[test]
    fn check_types_of_function_applications() {
        let module = Module::without_validation(
            vec![],
            vec![
                FunctionDefinition::new(
                    "f",
                    vec![Argument::new("x", types::Primitive::Float64)],
                    42.0,
                    types::Primitive::Float64,
                )
                .into(),
                ValueDefinition::new(
                    "x",
                    FunctionApplication::new(Variable::new("f"), vec![42.0.into()]),
                    types::Primitive::Float64,
                )
                .into(),
            ],
            vec![],
        );

        assert_eq!(check_types(&module), Ok(()));
    }

    #[test]
    fn fail_to_check_types_of_function_applications() {
        let module = Module::without_validation(
            vec![],
            vec![
                FunctionDefinition::new(
                    "f",
                    vec![Argument::new("x", types::Primitive::Float64)],
                    42.0,
                    types::Primitive::Float64,
                )
                .into(),
                ValueDefinition::new(
                    "x",
                    FunctionApplication::new(Variable::new("f"), vec![42.0.into(), 42.0.into()]),
                    types::Primitive::Float64,
                )
                .into(),
            ],
            vec![],
        );

        assert_eq!(check_types(&module), Err(TypeCheckError));
    }

    #[test]
    fn fail_to_check_types_because_of_missing_variables() {
        let module = Module::without_validation(
            vec![],
            vec![ValueDefinition::new("x", Variable::new("y"), types::Primitive::Float64).into()],
            vec![],
        );

        assert_eq!(check_types(&module), Err(TypeCheckError));
    }

    #[test]
    fn check_types_of_let_values() {
        let module = Module::without_validation(
            vec![],
            vec![ValueDefinition::new(
                "x",
                LetValues::new(
                    vec![
                        ValueDefinition::new("y", 42.0, types::Primitive::Float64),
                        ValueDefinition::new("z", Variable::new("y"), types::Primitive::Float64),
                    ],
                    Variable::new("z"),
                ),
                types::Primitive::Float64,
            )
            .into()],
            vec![],
        );

        assert_eq!(check_types(&module), Ok(()));
    }

    #[test]
    fn fail_to_check_types_of_let_values() {
        let module = Module::without_validation(
            vec![],
            vec![
                FunctionDefinition::new(
                    "f",
                    vec![Argument::new("x", types::Primitive::Float64)],
                    42.0,
                    types::Primitive::Float64,
                )
                .into(),
                ValueDefinition::new(
                    "x",
                    LetValues::new(
                        vec![ValueDefinition::new(
                            "y",
                            Variable::new("f"),
                            types::Primitive::Float64,
                        )],
                        Variable::new("y"),
                    ),
                    types::Primitive::Float64,
                )
                .into(),
            ],
            vec![],
        );

        assert_eq!(check_types(&module), Err(TypeCheckError));
    }

    #[test]
    fn check_types_of_declarations() {
        let module = Module::without_validation(
            vec![Declaration::new("x", types::Primitive::Float64)],
            vec![ValueDefinition::new("y", Variable::new("x"), types::Primitive::Float64).into()],
            vec![],
        );
        assert_eq!(check_types(&module), Ok(()));
    }

    #[test]
    fn fail_to_check_types_of_declarations() {
        let module = Module::without_validation(
            vec![Declaration::new(
                "x",
                types::Function::new(
                    vec![types::Primitive::Float64.into()],
                    types::Primitive::Float64,
                ),
            )],
            vec![ValueDefinition::new("y", Variable::new("x"), types::Primitive::Float64).into()],
            vec![],
        );
        assert_eq!(check_types(&module), Err(TypeCheckError));
    }

    mod case_expressions {
        use super::*;

        #[test]
        fn check_case_expressions_only_with_default_alternative() {
            assert_eq!(
                check_types(&Module::without_validation(
                    vec![],
                    vec![FunctionDefinition::new(
                        "f",
                        vec![Argument::new(
                            "x",
                            types::Algebraic::new(vec![types::Constructor::new(vec![])]),
                        )],
                        AlgebraicCase::new(
                            Variable::new("x"),
                            vec![],
                            Some(DefaultAlternative::new("x", 42.0)),
                        ),
                        types::Primitive::Float64,
                    )
                    .into()],
                    vec![],
                )),
                Ok(())
            );
        }

        #[test]
        fn check_case_expressions_only_with_default_alternative_and_bound_variable() {
            let algebraic_type = types::Algebraic::new(vec![types::Constructor::new(vec![])]);

            assert_eq!(
                check_types(&Module::without_validation(
                    vec![],
                    vec![FunctionDefinition::new(
                        "f",
                        vec![Argument::new("x", algebraic_type.clone())],
                        AlgebraicCase::new(
                            Variable::new("x"),
                            vec![],
                            Some(DefaultAlternative::new("y", Variable::new("y"))),
                        ),
                        algebraic_type,
                    )
                    .into()],
                    vec![],
                )),
                Ok(())
            );
        }

        #[test]
        fn check_case_expressions_with_one_alternative() {
            let algebraic_type = types::Algebraic::new(vec![types::Constructor::new(vec![])]);

            assert_eq!(
                check_types(&Module::without_validation(
                    vec![],
                    vec![FunctionDefinition::new(
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
                    )
                    .into()],
                    vec![],
                )),
                Ok(())
            );
        }

        #[test]
        fn check_case_expressions_with_deconstruction() {
            let algebraic_type = types::Algebraic::new(vec![types::Constructor::new(vec![
                types::Primitive::Float64.into(),
            ])]);

            assert_eq!(
                check_types(&Module::without_validation(
                    vec![],
                    vec![FunctionDefinition::new(
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
                    )
                    .into()],
                    vec![],
                )),
                Ok(())
            );
        }

        #[test]
        fn fail_to_check_case_expressions_without_alternatives() {
            assert_eq!(
                check_types(&Module::without_validation(
                    vec![],
                    vec![FunctionDefinition::new(
                        "f",
                        vec![Argument::new(
                            "x",
                            types::Algebraic::new(vec![types::Constructor::new(vec![])]),
                        )],
                        AlgebraicCase::new(Variable::new("x"), vec![], None),
                        types::Primitive::Float64,
                    )
                    .into()],
                    vec![],
                )),
                Err(TypeCheckError)
            );
        }

        #[test]
        fn fail_to_check_case_expressions_with_inconsistent_alternative_types() {
            let algebraic_type = types::Algebraic::new(vec![types::Constructor::new(vec![])]);

            assert_eq!(
                check_types(&Module::without_validation(
                    vec![],
                    vec![FunctionDefinition::new(
                        "f",
                        vec![Argument::new(
                            "x",
                            types::Algebraic::new(vec![types::Constructor::new(vec![])]),
                        )],
                        AlgebraicCase::new(
                            Variable::new("x"),
                            vec![
                                AlgebraicAlternative::new(
                                    Constructor::new(algebraic_type.clone(), 0),
                                    vec![],
                                    Variable::new("x")
                                ),
                                AlgebraicAlternative::new(
                                    Constructor::new(algebraic_type, 0),
                                    vec![],
                                    42.0
                                )
                            ],
                            None
                        ),
                        types::Primitive::Float64,
                    )
                    .into()],
                    vec![],
                )),
                Err(TypeCheckError)
            );
        }
    }

    mod constructor_applications {
        use super::*;

        #[test]
        fn check_constructor_applications_with_no_arguments() {
            let algebraic_type = types::Algebraic::new(vec![types::Constructor::new(vec![])]);

            assert_eq!(
                check_types(&Module::without_validation(
                    vec![],
                    vec![ValueDefinition::new(
                        "x",
                        ConstructorApplication::new(
                            Constructor::new(algebraic_type.clone(), 0),
                            vec![],
                        ),
                        algebraic_type,
                    )
                    .into()],
                    vec![],
                )),
                Ok(())
            );
        }

        #[test]
        fn check_constructor_applications_with_arguments() {
            let algebraic_type = types::Algebraic::new(vec![types::Constructor::new(vec![
                types::Primitive::Float64.into(),
            ])]);

            assert_eq!(
                check_types(&Module::without_validation(
                    vec![],
                    vec![ValueDefinition::new(
                        "x",
                        ConstructorApplication::new(
                            Constructor::new(algebraic_type.clone(), 0),
                            vec![42.0.into()],
                        ),
                        algebraic_type,
                    )
                    .into()],
                    vec![],
                )),
                Ok(())
            );
        }

        #[test]
        fn fail_to_check_constructor_applications_with_wrong_number_of_arguments() {
            let algebraic_type = types::Algebraic::new(vec![types::Constructor::new(vec![
                types::Primitive::Float64.into(),
            ])]);

            assert_eq!(
                check_types(&Module::without_validation(
                    vec![],
                    vec![ValueDefinition::new(
                        "x",
                        ConstructorApplication::new(
                            Constructor::new(algebraic_type.clone(), 0),
                            vec![42.0.into(), 42.0.into()],
                        ),
                        algebraic_type,
                    )
                    .into()],
                    vec![],
                )),
                Err(TypeCheckError)
            );
        }

        #[test]
        fn fail_to_check_constructor_applications_with_wrong_argument_type() {
            let algebraic_type = types::Algebraic::new(vec![types::Constructor::new(vec![
                types::Primitive::Float64.into(),
            ])]);

            assert_eq!(
                check_types(&Module::without_validation(
                    vec![],
                    vec![ValueDefinition::new(
                        "x",
                        ConstructorApplication::new(
                            Constructor::new(algebraic_type.clone(), 0),
                            vec![ConstructorApplication::new(
                                Constructor::new(algebraic_type.clone(), 0),
                                vec![42.0.into()],
                            )
                            .into()],
                        ),
                        algebraic_type,
                    )
                    .into()],
                    vec![],
                )),
                Err(TypeCheckError)
            );
        }
    }
}
