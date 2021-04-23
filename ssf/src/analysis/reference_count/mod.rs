use crate::ir::*;
use std::collections::HashSet;

pub fn reference_count(module: &Module) -> Module {
    Module::new(
        module.foreign_declarations().to_vec(),
        module.foreign_definitions().to_vec(),
        module.declarations().to_vec(),
        module
            .definitions()
            .iter()
            .map(convert_definition)
            .collect(),
    )
}

fn convert_definition(definition: &Definition) -> Definition {
    Definition::new(
        definition.name(),
        definition.arguments().to_vec(),
        convert_expression(definition.body(), &Default::default()).0,
        definition.result_type().clone(),
    )
}

fn convert_expression(
    expression: &Expression,
    variables: &HashSet<String>,
) -> (Expression, HashSet<String>) {
    match expression {
        Expression::ArithmeticOperation(operation) => {
            let (rhs, variables) = convert_expression(operation.rhs(), variables);
            let (lhs, variables) = convert_expression(operation.lhs(), &variables);

            (
                ArithmeticOperation::new(operation.operator(), lhs, rhs).into(),
                variables,
            )
        }
        Expression::Bitcast(bitcast) => {
            let (expression, variables) = convert_expression(bitcast.expression(), variables);

            (
                Bitcast::new(expression, bitcast.type_().clone()).into(),
                variables,
            )
        }
        Expression::Case(case) => {
            let (case, variables) = convert_case(case, variables);

            (case.into(), variables)
        }
        Expression::ComparisonOperation(operation) => {
            let (rhs, variables) = convert_expression(operation.rhs(), variables);
            let (lhs, variables) = convert_expression(operation.lhs(), &variables);

            (
                ComparisonOperation::new(operation.operator(), lhs, rhs).into(),
                variables,
            )
        }
        Expression::ConstructorApplication(application) => {
            let (arguments, variables) = application.arguments().iter().rev().fold(
                (vec![], variables.clone()),
                |(arguments, variables), argument| {
                    let (argument, variables) = convert_expression(argument, &variables);

                    (
                        arguments.into_iter().chain(vec![argument]).collect(),
                        variables,
                    )
                },
            );

            (
                ConstructorApplication::new(
                    application.constructor().clone(),
                    arguments.into_iter().rev().collect(),
                )
                .into(),
                variables,
            )
        }
        Expression::FunctionApplication(function_application) => {
            let (argument, variables) =
                convert_expression(function_application.argument(), variables);
            let (function, variables) =
                convert_expression(function_application.function(), &variables);

            (
                FunctionApplication::new(function, argument).into(),
                variables,
            )
        }
        Expression::LetRecursive(let_) => {
            // TODO Collect variables from environments of definitions.
            let (expression, variables) = convert_expression(let_.expression(), variables);

            (
                LetRecursive::new(
                    let_.definitions().iter().map(convert_definition).collect(),
                    expression,
                )
                .into(),
                variables,
            )
        }
        Expression::Let(let_) => {
            let (expression, variables) = convert_expression(let_.expression(), &variables);
            let (bound_expression, variables) =
                convert_expression(let_.bound_expression(), &variables);

            (
                Let::new(
                    let_.name(),
                    let_.type_().clone(),
                    bound_expression,
                    expression,
                )
                .into(),
                variables.clone(),
            )
        }
        Expression::Variable(_) => todo!("clone a variable if it is moved already"),
        Expression::Primitive(_) => (expression.clone(), variables.clone()),
    }
}

fn convert_case(case: &Case, variables: &HashSet<String>) -> (Case, HashSet<String>) {
    match case {
        Case::Algebraic(case) => {
            let alternatives = case
                .alternatives()
                .iter()
                .map(|alternative| convert_algebraic_alternative(alternative, variables))
                .collect::<Vec<_>>();
            let default_alternative = case
                .default_alternative()
                .map(|expression| convert_expression(expression, variables));
            let (argument, variables) = convert_expression(case.argument(), &variables);

            (
                AlgebraicCase::new(argument, todo!(), todo!()).into(),
                variables,
            )
        }
        Case::Primitive(case) => {
            let alternatives = case
                .alternatives()
                .iter()
                .map(|alternative| convert_primitive_alternative(alternative, variables))
                .collect::<Vec<_>>();
            let default_alternative = case
                .default_alternative()
                .map(|expression| convert_expression(expression, variables));
            let variables = alternatives
                .iter()
                .flat_map(|(_, variables)| variables.clone())
                .chain(
                    default_alternative
                        .map(|(_, variables)| variables.clone())
                        .unwrap_or_default(),
                )
                .collect();
            let (argument, variables) = convert_expression(case.argument(), &variables);

            (
                PrimitiveCase::new(argument, todo!(), todo!()).into(),
                variables,
            )
        }
    }
}

fn convert_algebraic_alternative(
    alternative: &AlgebraicAlternative,
    variables: &HashSet<String>,
) -> (AlgebraicAlternative, HashSet<String>) {
    let (expression, variables) = convert_expression(alternative.expression(), variables);

    (
        AlgebraicAlternative::new(
            alternative.constructor().clone(),
            alternative.element_names().to_vec(),
            expression,
        ),
        variables,
    )
}

fn convert_primitive_alternative(
    alternative: &PrimitiveAlternative,
    variables: &HashSet<String>,
) -> (PrimitiveAlternative, HashSet<String>) {
    let (expression, variables) = convert_expression(alternative.expression(), variables);

    (
        PrimitiveAlternative::new(alternative.primitive().clone(), expression),
        variables,
    )
}
