use crate::ir::*;

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
        convert_expression(definition.body()),
        definition.result_type().clone(),
    )
}

fn convert_expression(expression: &Expression) -> Expression {
    match expression {
        Expression::ArithmeticOperation(operation) => ArithmeticOperation::new(
            operation.operator(),
            convert_expression(operation.lhs()),
            convert_expression(operation.rhs()),
        )
        .into(),
        Expression::Bitcast(bitcast) => Bitcast::new(
            convert_expression(bitcast.expression()),
            bitcast.type_().clone(),
        )
        .into(),
        Expression::Case(case) => convert_case(case).into(),
        Expression::ComparisonOperation(operation) => ComparisonOperation::new(
            operation.operator(),
            convert_expression(operation.lhs()),
            convert_expression(operation.rhs()),
        )
        .into(),
        Expression::ConstructorApplication(application) => ConstructorApplication::new(
            application.constructor().clone(),
            application
                .arguments()
                .iter()
                .map(|argument| convert_expression(argument))
                .collect(),
        )
        .into(),
        Expression::FunctionApplication(function_application) => FunctionApplication::new(
            convert_expression(function_application.function()),
            convert_expression(function_application.argument()),
        )
        .into(),
        Expression::LetRecursive(let_) => LetRecursive::new(
            let_.definitions().iter().map(convert_definition).collect(),
            convert_expression(let_.expression()),
        )
        .into(),
        Expression::Let(let_) => Let::new(
            let_.name(),
            let_.type_().clone(),
            convert_expression(let_.bound_expression()),
            convert_expression(let_.expression()),
        )
        .into(),
        Expression::Variable(variable) => todo!(),
        Expression::Primitive(_) => expression.clone(),
    }
}

fn convert_case(case: &Case) -> Case {
    match case {
        Case::Algebraic(case) => AlgebraicCase::new(
            convert_expression(case.argument()),
            case.alternatives()
                .iter()
                .map(convert_algebraic_alternative)
                .collect(),
            case.default_alternative().map(convert_expression),
        )
        .into(),
        Case::Primitive(case) => PrimitiveCase::new(
            convert_expression(case.argument()),
            case.alternatives()
                .iter()
                .map(convert_primitive_alternative)
                .collect(),
            case.default_alternative().map(convert_expression),
        )
        .into(),
    }
}

fn convert_algebraic_alternative(alternative: &AlgebraicAlternative) -> AlgebraicAlternative {
    AlgebraicAlternative::new(
        alternative.constructor().clone(),
        alternative.element_names().to_vec(),
        convert_expression(alternative.expression()),
    )
}

fn convert_primitive_alternative(alternative: &PrimitiveAlternative) -> PrimitiveAlternative {
    PrimitiveAlternative::new(
        alternative.primitive().clone(),
        convert_expression(alternative.expression()),
    )
}
