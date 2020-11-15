use super::algebraic_case::AlgebraicCase;
use super::bitcast::Bitcast;
use super::case::Case;
use super::constructor_application::ConstructorApplication;
use super::function_application::FunctionApplication;
use super::let_::Let;
use super::let_recursive::LetRecursive;
use super::operation::Operation;
use super::primitive::Primitive;
use super::primitive_case::PrimitiveCase;
use super::variable::Variable;
use crate::types::Type;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    Bitcast(Bitcast),
    Case(Case),
    ConstructorApplication(ConstructorApplication),
    FunctionApplication(FunctionApplication),
    LetRecursive(LetRecursive),
    Let(Let),
    Primitive(Primitive),
    Operation(Operation),
    Variable(Variable),
}

impl Expression {
    pub fn to_variable(&self) -> Option<&Variable> {
        match self {
            Self::Variable(variable) => Some(variable),
            _ => None,
        }
    }

    pub(crate) fn find_variables(&self) -> HashSet<String> {
        match self {
            Self::Bitcast(bitcast) => bitcast.find_variables(),
            Self::Case(case) => case.find_variables(),
            Self::ConstructorApplication(constructor_application) => {
                constructor_application.find_variables()
            }
            Self::FunctionApplication(function_application) => {
                function_application.find_variables()
            }
            Self::LetRecursive(let_recursive) => let_recursive.find_variables(),
            Self::Let(let_) => let_.find_variables(),
            Self::Operation(operation) => operation.find_variables(),
            Self::Variable(variable) => variable.find_variables(),
            Self::Primitive(_) => HashSet::new(),
        }
    }

    pub(crate) fn infer_environment(&self, variables: &HashMap<String, Type>) -> Self {
        match self {
            Self::Bitcast(bitcast) => bitcast.infer_environment(variables).into(),
            Self::Case(case) => case.infer_environment(variables).into(),
            Self::ConstructorApplication(constructor_application) => {
                constructor_application.infer_environment(variables).into()
            }
            Self::FunctionApplication(function_application) => {
                function_application.infer_environment(variables).into()
            }
            Self::LetRecursive(let_recursive) => let_recursive.infer_environment(variables).into(),
            Self::Let(let_) => let_.infer_environment(variables).into(),
            Self::Operation(operation) => operation.infer_environment(variables).into(),
            Self::Primitive(_) | Self::Variable(_) => self.clone(),
        }
    }

    pub(crate) fn convert_types(&self, convert: &impl Fn(&Type) -> Type) -> Self {
        match self {
            Self::Bitcast(bitcast) => bitcast.convert_types(convert).into(),
            Self::Case(case) => case.convert_types(convert).into(),
            Self::ConstructorApplication(constructor_application) => {
                constructor_application.convert_types(convert).into()
            }
            Self::FunctionApplication(function_application) => {
                function_application.convert_types(convert).into()
            }
            Self::LetRecursive(let_recursive) => let_recursive.convert_types(convert).into(),
            Self::Let(let_) => let_.convert_types(convert).into(),
            Self::Operation(operation) => operation.convert_types(convert).into(),
            Self::Primitive(_) | Self::Variable(_) => self.clone(),
        }
    }
}

impl From<AlgebraicCase> for Expression {
    fn from(algebraic_case: AlgebraicCase) -> Self {
        Self::Case(algebraic_case.into())
    }
}

impl From<Bitcast> for Expression {
    fn from(bitcast: Bitcast) -> Self {
        Self::Bitcast(bitcast)
    }
}

impl From<Case> for Expression {
    fn from(case: Case) -> Self {
        Self::Case(case)
    }
}

impl From<ConstructorApplication> for Expression {
    fn from(constructor_application: ConstructorApplication) -> Self {
        Self::ConstructorApplication(constructor_application)
    }
}

impl From<FunctionApplication> for Expression {
    fn from(function_application: FunctionApplication) -> Self {
        Self::FunctionApplication(function_application)
    }
}

impl From<LetRecursive> for Expression {
    fn from(let_recursive: LetRecursive) -> Self {
        Self::LetRecursive(let_recursive)
    }
}

impl From<Let> for Expression {
    fn from(let_: Let) -> Self {
        Self::Let(let_)
    }
}

impl From<Operation> for Expression {
    fn from(operation: Operation) -> Self {
        Self::Operation(operation)
    }
}

impl<T: Into<Primitive>> From<T> for Expression {
    fn from(primitive: T) -> Self {
        Self::Primitive(primitive.into())
    }
}

impl From<PrimitiveCase> for Expression {
    fn from(primitive_case: PrimitiveCase) -> Self {
        Self::Case(primitive_case.into())
    }
}

impl From<Variable> for Expression {
    fn from(variable: Variable) -> Self {
        Self::Variable(variable)
    }
}
