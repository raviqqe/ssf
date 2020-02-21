use super::algebraic_case::AlgebraicCase;
use super::case::Case;
use super::constructor_application::ConstructorApplication;
use super::function_application::FunctionApplication;
use super::let_functions::LetFunctions;
use super::let_values::LetValues;
use super::literal::Literal;
use super::operation::Operation;
use super::variable::Variable;
use crate::types::Type;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    Case(Case),
    ConstructorApplication(ConstructorApplication),
    FunctionApplication(FunctionApplication),
    LetFunctions(LetFunctions),
    LetValues(LetValues),
    Literal(Literal),
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

    pub(crate) fn rename_variables(&self, names: &HashMap<String, String>) -> Self {
        match self {
            Self::Case(case) => case.rename_variables(names).into(),
            Self::ConstructorApplication(constructor_application) => {
                constructor_application.rename_variables(names).into()
            }
            Self::FunctionApplication(function_application) => {
                function_application.rename_variables(names).into()
            }
            Self::LetFunctions(let_functions) => let_functions.rename_variables(names).into(),
            Self::LetValues(let_values) => let_values.rename_variables(names).into(),
            Self::Operation(operation) => operation.rename_variables(names).into(),
            Self::Variable(variable) => variable.rename_variables(names).into(),
            Self::Literal(_) => self.clone(),
        }
    }

    pub(crate) fn find_variables(&self, excluded_variables: &HashSet<String>) -> HashSet<String> {
        match self {
            Self::Case(case) => case.find_variables(excluded_variables),
            Self::ConstructorApplication(constructor_application) => {
                constructor_application.find_variables(excluded_variables)
            }
            Self::FunctionApplication(function_application) => {
                function_application.find_variables(excluded_variables)
            }
            Self::LetFunctions(let_functions) => let_functions.find_variables(excluded_variables),
            Self::LetValues(let_values) => let_values.find_variables(excluded_variables),
            Self::Operation(operation) => operation.find_variables(excluded_variables),
            Self::Variable(variable) => variable.find_variables(excluded_variables),
            Self::Literal(_) => HashSet::new(),
        }
    }

    pub(crate) fn infer_environment(
        &self,
        variables: &HashMap<String, Type>,
        global_variables: &HashSet<String>,
    ) -> Self {
        match self {
            Self::Case(case) => case.infer_environment(variables, global_variables).into(),
            Self::ConstructorApplication(constructor_application) => constructor_application
                .infer_environment(variables, global_variables)
                .into(),
            Self::FunctionApplication(function_application) => function_application
                .infer_environment(variables, global_variables)
                .into(),
            Self::LetFunctions(let_functions) => let_functions
                .infer_environment(variables, global_variables)
                .into(),
            Self::LetValues(let_values) => let_values
                .infer_environment(variables, global_variables)
                .into(),
            Self::Operation(operation) => operation
                .infer_environment(variables, global_variables)
                .into(),
            Self::Literal(_) | Self::Variable(_) => self.clone(),
        }
    }

    pub(crate) fn convert_types(&self, convert: &impl Fn(&Type) -> Type) -> Self {
        match self {
            Self::Case(case) => case.convert_types(convert).into(),
            Self::ConstructorApplication(constructor_application) => {
                constructor_application.convert_types(convert).into()
            }
            Self::FunctionApplication(function_application) => {
                function_application.convert_types(convert).into()
            }
            Self::LetFunctions(let_functions) => let_functions.convert_types(convert).into(),
            Self::LetValues(let_values) => let_values.convert_types(convert).into(),
            Self::Operation(operation) => operation.convert_types(convert).into(),
            Self::Literal(_) | Self::Variable(_) => self.clone(),
        }
    }
}

impl From<AlgebraicCase> for Expression {
    fn from(algebraic_case: AlgebraicCase) -> Expression {
        Self::Case(algebraic_case.into())
    }
}

impl From<Case> for Expression {
    fn from(case: Case) -> Expression {
        Self::Case(case)
    }
}

impl From<ConstructorApplication> for Expression {
    fn from(constructor_application: ConstructorApplication) -> Expression {
        Self::ConstructorApplication(constructor_application)
    }
}

impl From<FunctionApplication> for Expression {
    fn from(function_application: FunctionApplication) -> Expression {
        Self::FunctionApplication(function_application)
    }
}

impl From<LetFunctions> for Expression {
    fn from(let_functions: LetFunctions) -> Expression {
        Self::LetFunctions(let_functions)
    }
}

impl From<LetValues> for Expression {
    fn from(let_values: LetValues) -> Expression {
        Self::LetValues(let_values)
    }
}

impl<T: Into<Literal>> From<T> for Expression {
    fn from(literal: T) -> Self {
        Self::Literal(literal.into())
    }
}

impl From<Operation> for Expression {
    fn from(operation: Operation) -> Expression {
        Self::Operation(operation)
    }
}

impl From<Variable> for Expression {
    fn from(variable: Variable) -> Expression {
        Self::Variable(variable)
    }
}
