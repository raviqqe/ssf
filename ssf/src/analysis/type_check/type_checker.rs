use super::error::TypeCheckError;
use crate::ir::*;
use crate::types::{self, Type};
use std::collections::*;

const ARRAY_INDEX_TYPE: types::Primitive = types::Primitive::Integer64;

#[derive(Clone, Debug)]
pub struct TypeChecker {}

impl TypeChecker {
    pub fn new() -> Self {
        Self {}
    }

    pub fn check(&self, module: &Module) -> Result<(), TypeCheckError> {
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
            self.check_definition(definition, &variables)?;
        }

        Ok(())
    }

    fn check_definition(
        &self,
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

        self.check_equality(
            &self.check_expression(definition.body(), &variables)?,
            &definition.result_type().clone(),
        )
    }

    fn check_expression(
        &self,
        expression: &Expression,
        variables: &HashMap<&str, Type>,
    ) -> Result<Type, TypeCheckError> {
        Ok(match expression {
            Expression::Array(array) => {
                for element in array.elements() {
                    match element {
                        ArrayElement::Multiple(element) => {
                            self.check_equality(
                                &self.check_expression(element.array(), variables)?,
                                &types::Array::new(array.element_type().clone()).into(),
                            )?;

                            self.check_equality(
                                &self.check_expression(element.length(), variables)?,
                                &ARRAY_INDEX_TYPE.into(),
                            )?;
                        }
                        ArrayElement::Single(element) => {
                            self.check_equality(
                                &self.check_expression(element, variables)?,
                                array.element_type(),
                            )?;
                        }
                    }
                }

                types::Array::new(array.element_type().clone()).into()
            }
            Expression::ArrayGetOperation(operation) => {
                let type_ = self.check_expression(operation.array(), variables)?;

                if let Type::Array(array_type) = type_ {
                    let type_ = self.check_expression(operation.index(), variables)?;

                    if type_ != ARRAY_INDEX_TYPE.into() {
                        return Err(TypeCheckError::TypesNotMatched(
                            type_,
                            ARRAY_INDEX_TYPE.into(),
                        ));
                    }

                    array_type.element().clone()
                } else {
                    return Err(TypeCheckError::ArrayExpected(type_));
                }
            }
            Expression::Bitcast(bitcast) => {
                self.check_expression(bitcast.expression(), variables)?;
                bitcast.type_().clone()
            }
            Expression::Case(case) => self.check_case(case, variables)?,
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
                    self.check_equality(
                        &self.check_expression(argument, variables)?,
                        &element_type,
                    )?;
                }

                constructor_application
                    .constructor()
                    .algebraic_type()
                    .clone()
                    .into()
            }
            Expression::FunctionApplication(function_application) => {
                if let Type::Function(function_type) =
                    self.check_expression(function_application.function(), variables)?
                {
                    self.check_equality(
                        &self.check_expression(function_application.argument(), variables)?,
                        function_type.argument(),
                    )?;

                    function_type.result().clone()
                } else {
                    return Err(TypeCheckError::FunctionExpected(
                        function_application.function().clone(),
                    ));
                }
            }
            Expression::LetRecursive(let_recursive) => {
                let mut variables = variables.clone();

                for definition in let_recursive.definitions() {
                    variables.insert(definition.name(), definition.type_().clone().into());
                }

                for definition in let_recursive.definitions() {
                    self.check_definition(definition, &variables)?;
                }

                self.check_expression(let_recursive.expression(), &variables)?
            }
            Expression::Let(let_) => {
                self.check_equality(
                    &self.check_expression(let_.bound_expression(), variables)?,
                    let_.type_(),
                )?;

                let mut variables = variables.clone();
                variables.insert(let_.name(), let_.type_().clone());

                self.check_expression(let_.expression(), &variables)?
            }
            Expression::Primitive(primitive) => Ok(self.check_primitive(primitive).into())?,
            Expression::PrimitiveOperation(operation) => {
                let lhs_type = self.check_expression(operation.lhs(), variables)?;
                let rhs_type = self.check_expression(operation.rhs(), variables)?;

                if !lhs_type.is_primitive() || !rhs_type.is_primitive() || lhs_type != rhs_type {
                    return Err(TypeCheckError::TypesNotMatched(lhs_type, rhs_type));
                }

                match operation.operator() {
                    PrimitiveOperator::Equal
                    | PrimitiveOperator::NotEqual
                    | PrimitiveOperator::GreaterThan
                    | PrimitiveOperator::GreaterThanOrEqual
                    | PrimitiveOperator::LessThan
                    | PrimitiveOperator::LessThanOrEqual => types::Primitive::Integer8.into(),
                    PrimitiveOperator::Add
                    | PrimitiveOperator::Subtract
                    | PrimitiveOperator::Multiply
                    | PrimitiveOperator::Divide => lhs_type,
                }
            }
            Expression::Variable(variable) => self.check_variable(variable, variables)?,
        })
    }

    fn check_case(
        &self,
        case: &Case,
        variables: &HashMap<&str, Type>,
    ) -> Result<Type, TypeCheckError> {
        match case {
            Case::Algebraic(algebraic_case) => {
                let argument_type = self.check_expression(algebraic_case.argument(), variables)?;
                let mut expression_type = None;

                for alternative in algebraic_case.alternatives() {
                    let constructor = alternative.constructor();

                    self.check_equality(
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

                    let alternative_type =
                        self.check_expression(alternative.expression(), &variables)?;

                    if let Some(expression_type) = &expression_type {
                        self.check_equality(&alternative_type, expression_type)?;
                    } else {
                        expression_type = Some(alternative_type);
                    }
                }

                if let Some(expression) = algebraic_case.default_alternative() {
                    let alternative_type = self.check_expression(expression, &variables)?;

                    if let Some(expression_type) = &expression_type {
                        self.check_equality(&alternative_type, expression_type)?;
                    } else {
                        expression_type = Some(alternative_type);
                    }
                }

                expression_type.ok_or_else(|| TypeCheckError::NoAlternativeFound(case.clone()))
            }
            Case::Primitive(primitive_case) => {
                let argument_type = self.check_expression(primitive_case.argument(), variables)?;
                let mut expression_type = None;

                for alternative in primitive_case.alternatives() {
                    self.check_equality(
                        &self.check_primitive(alternative.primitive()).into(),
                        &argument_type.clone(),
                    )?;

                    let alternative_type =
                        self.check_expression(alternative.expression(), variables)?;

                    if let Some(expression_type) = &expression_type {
                        self.check_equality(&alternative_type, expression_type)?;
                    } else {
                        expression_type = Some(alternative_type);
                    }
                }

                if let Some(expression) = primitive_case.default_alternative() {
                    let alternative_type = self.check_expression(expression, &variables)?;

                    if let Some(expression_type) = &expression_type {
                        self.check_equality(&alternative_type, expression_type)?;
                    } else {
                        expression_type = Some(alternative_type);
                    }
                }

                expression_type.ok_or_else(|| TypeCheckError::NoAlternativeFound(case.clone()))
            }
        }
    }

    fn check_primitive(&self, primitive: &Primitive) -> types::Primitive {
        match primitive {
            Primitive::Float32(_) => types::Primitive::Float32,
            Primitive::Float64(_) => types::Primitive::Float64,
            Primitive::Integer8(_) => types::Primitive::Integer8,
            Primitive::Integer32(_) => types::Primitive::Integer32,
            Primitive::Integer64(_) => types::Primitive::Integer64,
        }
    }

    fn check_variable(
        &self,
        variable: &Variable,
        variables: &HashMap<&str, Type>,
    ) -> Result<Type, TypeCheckError> {
        variables
            .get(variable.name())
            .cloned()
            .ok_or_else(|| TypeCheckError::VariableNotFound(variable.clone()))
    }

    fn check_equality(&self, one: &Type, other: &Type) -> Result<(), TypeCheckError> {
        if one == other {
            Ok(())
        } else {
            Err(TypeCheckError::TypesNotMatched(one.clone(), other.clone()))
        }
    }
}
