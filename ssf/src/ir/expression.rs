use super::{
    algebraic_case::AlgebraicCase, arithmetic_operation::ArithmeticOperation, bit_cast::BitCast,
    case::Case, comparison_operation::ComparisonOperation,
    constructor_application::ConstructorApplication, function_application::FunctionApplication,
    let_::Let, let_recursive::LetRecursive, primitive::Primitive, primitive_case::PrimitiveCase,
    variable::Variable,
};
use crate::types::Type;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    ArithmeticOperation(ArithmeticOperation),
    BitCast(BitCast),
    Case(Case),
    ComparisonOperation(ComparisonOperation),
    ConstructorApplication(ConstructorApplication),
    FunctionApplication(FunctionApplication),
    Let(Let),
    LetRecursive(LetRecursive),
    Primitive(Primitive),
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
            Self::ArithmeticOperation(operation) => operation.find_variables(),
            Self::BitCast(bit_cast) => bit_cast.find_variables(),
            Self::Case(case) => case.find_variables(),
            Self::ComparisonOperation(operation) => operation.find_variables(),
            Self::ConstructorApplication(constructor_application) => {
                constructor_application.find_variables()
            }
            Self::FunctionApplication(function_application) => {
                function_application.find_variables()
            }
            Self::LetRecursive(let_recursive) => let_recursive.find_variables(),
            Self::Let(let_) => let_.find_variables(),
            Self::Variable(variable) => variable.find_variables(),
            Self::Primitive(_) => HashSet::new(),
        }
    }

    pub(crate) fn infer_environment(&self, variables: &HashMap<String, Type>) -> Self {
        match self {
            Self::ArithmeticOperation(operation) => operation.infer_environment(variables).into(),
            Self::BitCast(bit_cast) => bit_cast.infer_environment(variables).into(),
            Self::Case(case) => case.infer_environment(variables).into(),
            Self::ComparisonOperation(operation) => operation.infer_environment(variables).into(),
            Self::ConstructorApplication(constructor_application) => {
                constructor_application.infer_environment(variables).into()
            }
            Self::FunctionApplication(function_application) => {
                function_application.infer_environment(variables).into()
            }
            Self::LetRecursive(let_recursive) => let_recursive.infer_environment(variables).into(),
            Self::Let(let_) => let_.infer_environment(variables).into(),
            Self::Primitive(_) | Self::Variable(_) => self.clone(),
        }
    }

    pub(crate) fn convert_types(&self, convert: &impl Fn(&Type) -> Type) -> Self {
        match self {
            Self::ArithmeticOperation(operation) => operation.convert_types(convert).into(),
            Self::BitCast(bit_cast) => bit_cast.convert_types(convert).into(),
            Self::Case(case) => case.convert_types(convert).into(),
            Self::ComparisonOperation(operation) => operation.convert_types(convert).into(),
            Self::ConstructorApplication(constructor_application) => {
                constructor_application.convert_types(convert).into()
            }
            Self::FunctionApplication(function_application) => {
                function_application.convert_types(convert).into()
            }
            Self::LetRecursive(let_recursive) => let_recursive.convert_types(convert).into(),
            Self::Let(let_) => let_.convert_types(convert).into(),
            Self::Primitive(_) | Self::Variable(_) => self.clone(),
        }
    }
}

impl From<AlgebraicCase> for Expression {
    fn from(algebraic_case: AlgebraicCase) -> Self {
        Self::Case(algebraic_case.into())
    }
}

impl From<ArithmeticOperation> for Expression {
    fn from(operation: ArithmeticOperation) -> Self {
        Self::ArithmeticOperation(operation)
    }
}

impl From<BitCast> for Expression {
    fn from(bit_cast: BitCast) -> Self {
        Self::BitCast(bit_cast)
    }
}

impl From<Case> for Expression {
    fn from(case: Case) -> Self {
        Self::Case(case)
    }
}

impl From<ComparisonOperation> for Expression {
    fn from(operation: ComparisonOperation) -> Self {
        Self::ComparisonOperation(operation)
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
