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
            vec![ValueDefinition::new("x", 42.0, types::Value::Number).into()],
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
                    vec![Argument::new("x", types::Value::Number)],
                    42.0,
                    types::Value::Number,
                )
                .into(),
                ValueDefinition::new("x", Variable::new("f"), types::Value::Number).into(),
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
                vec![Argument::new("x", types::Value::Number)],
                42.0,
                types::Value::Number,
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
                    vec![Argument::new("x", types::Value::Number)],
                    42.0,
                    types::Value::Number,
                )
                .into(),
                FunctionDefinition::new(
                    "g",
                    vec![Argument::new("x", types::Value::Number)],
                    Variable::new("f"),
                    types::Value::Number,
                )
                .into(),
            ],
            vec![],
        );

        assert_eq!(check_types(&module), Err(TypeCheckError));
    }

    #[test]
    fn check_types_of_applications() {
        let module = Module::without_validation(
            vec![],
            vec![
                FunctionDefinition::new(
                    "f",
                    vec![Argument::new("x", types::Value::Number)],
                    42.0,
                    types::Value::Number,
                )
                .into(),
                ValueDefinition::new(
                    "x",
                    Application::new(Variable::new("f"), vec![Expression::Number(42.0)]),
                    types::Value::Number,
                )
                .into(),
            ],
            vec![],
        );

        assert_eq!(check_types(&module), Ok(()));
    }

    #[test]
    fn fail_to_check_types_of_applications() {
        let module = Module::without_validation(
            vec![],
            vec![
                FunctionDefinition::new(
                    "f",
                    vec![Argument::new("x", types::Value::Number)],
                    42.0,
                    types::Value::Number,
                )
                .into(),
                ValueDefinition::new(
                    "x",
                    Application::new(
                        Variable::new("f"),
                        vec![Expression::Number(42.0), Expression::Number(42.0)],
                    ),
                    types::Value::Number,
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
            vec![ValueDefinition::new("x", Variable::new("y"), types::Value::Number).into()],
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
                        ValueDefinition::new("y", 42.0, types::Value::Number),
                        ValueDefinition::new("z", Variable::new("y"), types::Value::Number),
                    ],
                    Variable::new("z"),
                ),
                types::Value::Number,
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
                    vec![Argument::new("x", types::Value::Number)],
                    42.0,
                    types::Value::Number,
                )
                .into(),
                ValueDefinition::new(
                    "x",
                    LetValues::new(
                        vec![ValueDefinition::new(
                            "y",
                            Variable::new("f"),
                            types::Value::Number,
                        )],
                        Variable::new("y"),
                    ),
                    types::Value::Number,
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
            vec![Declaration::new("x", types::Value::Number)],
            vec![ValueDefinition::new("y", Variable::new("x"), types::Value::Number).into()],
            vec![],
        );
        assert_eq!(check_types(&module), Ok(()));
    }

    #[test]
    fn fail_to_check_types_of_declarations() {
        let module = Module::without_validation(
            vec![Declaration::new(
                "x",
                types::Function::new(vec![types::Value::Number.into()], types::Value::Number),
            )],
            vec![ValueDefinition::new("y", Variable::new("x"), types::Value::Number).into()],
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
                        types::Value::Number,
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
                        types::Value::Number,
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
                types::Value::Number.into(),
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
                        types::Value::Number,
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
                        types::Value::Number,
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
                        types::Value::Number,
                    )
                    .into()],
                    vec![],
                )),
                Err(TypeCheckError)
            );
        }
    }
}
