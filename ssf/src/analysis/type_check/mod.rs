mod error;

use crate::ir::*;
use crate::types::{self, Type};
pub use error::TypeCheckError;
use std::collections::*;

pub fn check_types(module: &Module) -> Result<(), TypeCheckError> {
    let mut variables = HashMap::<&str, Type>::new();

    for declaration in module.foreign_declarations() {
        variables.insert(declaration.name(), declaration.type_().clone().into());
    }

    for declaration in module.declarations() {
        variables.insert(declaration.name(), declaration.type_().clone().into());
    }

    for definition in module.definitions() {
        variables.insert(definition.name(), definition.type_().clone().into());
    }

    for definition in module.definitions() {
        check_definition(definition, &variables)?;
    }

    Ok(())
}

fn check_definition(
    definition: &Definition,
    variables: &HashMap<&str, Type>,
) -> Result<(), TypeCheckError> {
    let mut variables = variables.clone();

    for argument in definition
        .environment()
        .iter()
        .chain(definition.arguments())
    {
        variables.insert(argument.name(), argument.type_().clone());
    }

    check_equality(
        &check_expression(definition.body(), &variables)?,
        &definition.result_type().clone(),
    )
}

fn check_expression(
    expression: &Expression,
    variables: &HashMap<&str, Type>,
) -> Result<Type, TypeCheckError> {
    Ok(match expression {
        Expression::Bitcast(bitcast) => {
            check_expression(bitcast.expression(), variables)?;
            bitcast.type_().clone()
        }
        Expression::Case(case) => check_case(case, variables)?,
        Expression::ConstructorApplication(constructor_application) => {
            let constructor = constructor_application.constructor();

            if constructor_application.arguments().len()
                != constructor.constructor_type().elements().len()
            {
                return Err(TypeCheckError::WrongArgumentsLength(expression.clone()));
            }

            for (argument, element_type) in constructor_application
                .arguments()
                .iter()
                .zip(constructor.constructor_type().elements())
            {
                check_equality(&check_expression(argument, variables)?, &element_type)?;
            }

            constructor_application
                .constructor()
                .algebraic_type()
                .clone()
                .into()
        }
        Expression::FunctionApplication(function_application) => {
            let function_type = check_expression(function_application.function(), variables)?
                .into_function()
                .ok_or_else(|| {
                    TypeCheckError::FunctionExpected(function_application.function().clone())
                })?;

            check_equality(
                &check_expression(function_application.argument(), variables)?,
                function_type.argument(),
            )?;

            function_type.result().clone()
        }
        Expression::LetRecursive(let_recursive) => {
            let mut variables = variables.clone();

            for definition in let_recursive.definitions() {
                variables.insert(definition.name(), definition.type_().clone().into());
            }

            for definition in let_recursive.definitions() {
                check_definition(definition, &variables)?;
            }

            check_expression(let_recursive.expression(), &variables)?
        }
        Expression::Let(let_) => {
            check_equality(
                &check_expression(let_.bound_expression(), variables)?,
                let_.type_(),
            )?;

            let mut variables = variables.clone();
            variables.insert(let_.name(), let_.type_().clone());

            check_expression(let_.expression(), &variables)?
        }
        Expression::Primitive(primitive) => Ok(check_primitive(primitive).into())?,
        Expression::PrimitiveOperation(operation) => {
            let lhs_type = check_expression(operation.lhs(), variables)?;
            let rhs_type = check_expression(operation.rhs(), variables)?;

            if !lhs_type.is_primitive() || !rhs_type.is_primitive() || lhs_type != rhs_type {
                return Err(TypeCheckError::TypesNotMatched(lhs_type, rhs_type));
            }

            match operation.operator() {
                PrimitiveOperator::Equal
                | PrimitiveOperator::NotEqual
                | PrimitiveOperator::GreaterThan
                | PrimitiveOperator::GreaterThanOrEqual
                | PrimitiveOperator::LessThan
                | PrimitiveOperator::LessThanOrEqual => types::Primitive::Boolean.into(),
                PrimitiveOperator::Add
                | PrimitiveOperator::Subtract
                | PrimitiveOperator::Multiply
                | PrimitiveOperator::Divide => lhs_type,
            }
        }
        Expression::Variable(variable) => check_variable(variable, variables)?,
    })
}

fn check_case(case: &Case, variables: &HashMap<&str, Type>) -> Result<Type, TypeCheckError> {
    match case {
        Case::Algebraic(algebraic_case) => {
            let argument_type = check_expression(algebraic_case.argument(), variables)?;
            let mut expression_type = None;

            for alternative in algebraic_case.alternatives() {
                let constructor = alternative.constructor();

                check_equality(
                    &constructor.algebraic_type().clone().into(),
                    &argument_type.clone(),
                )?;

                let mut variables = variables.clone();

                for (name, type_) in alternative
                    .element_names()
                    .iter()
                    .zip(constructor.constructor_type().elements())
                {
                    variables.insert(name, type_.clone());
                }

                let alternative_type = check_expression(alternative.expression(), &variables)?;

                if let Some(expression_type) = &expression_type {
                    check_equality(&alternative_type, expression_type)?;
                } else {
                    expression_type = Some(alternative_type);
                }
            }

            if let Some(expression) = algebraic_case.default_alternative() {
                let alternative_type = check_expression(expression, &variables)?;

                if let Some(expression_type) = &expression_type {
                    check_equality(&alternative_type, expression_type)?;
                } else {
                    expression_type = Some(alternative_type);
                }
            }

            expression_type.ok_or_else(|| TypeCheckError::NoAlternativeFound(case.clone()))
        }
        Case::Primitive(primitive_case) => {
            let argument_type = check_expression(primitive_case.argument(), variables)?;
            let mut expression_type = None;

            for alternative in primitive_case.alternatives() {
                check_equality(
                    &check_primitive(alternative.primitive()).into(),
                    &argument_type.clone(),
                )?;

                let alternative_type = check_expression(alternative.expression(), variables)?;

                if let Some(expression_type) = &expression_type {
                    check_equality(&alternative_type, expression_type)?;
                } else {
                    expression_type = Some(alternative_type);
                }
            }

            if let Some(expression) = primitive_case.default_alternative() {
                let alternative_type = check_expression(expression, &variables)?;

                if let Some(expression_type) = &expression_type {
                    check_equality(&alternative_type, expression_type)?;
                } else {
                    expression_type = Some(alternative_type);
                }
            }

            expression_type.ok_or_else(|| TypeCheckError::NoAlternativeFound(case.clone()))
        }
    }
}

fn check_primitive(primitive: &Primitive) -> types::Primitive {
    match primitive {
        Primitive::Boolean(_) => types::Primitive::Boolean,
        Primitive::Float32(_) => types::Primitive::Float32,
        Primitive::Float64(_) => types::Primitive::Float64,
        Primitive::Integer8(_) => types::Primitive::Integer8,
        Primitive::Integer32(_) => types::Primitive::Integer32,
        Primitive::Integer64(_) => types::Primitive::Integer64,
    }
}

fn check_variable(
    variable: &Variable,
    variables: &HashMap<&str, Type>,
) -> Result<Type, TypeCheckError> {
    variables
        .get(variable.name())
        .cloned()
        .ok_or_else(|| TypeCheckError::VariableNotFound(variable.clone()))
}

fn check_equality(one: &Type, other: &Type) -> Result<(), TypeCheckError> {
    if one == other {
        Ok(())
    } else {
        Err(TypeCheckError::TypesNotMatched(one.clone(), other.clone()))
    }
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
                types::Primitive::Boolean,
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
