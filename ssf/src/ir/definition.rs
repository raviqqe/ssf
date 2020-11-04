use super::function_definition::*;
use super::value_definition::*;
use crate::types::Type;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq)]
pub enum Definition {
    FunctionDefinition(FunctionDefinition),
    ValueDefinition(ValueDefinition),
}

impl Definition {
    pub fn name(&self) -> &str {
        match self {
            Self::FunctionDefinition(function_definition) => function_definition.name(),
            Self::ValueDefinition(value_definition) => value_definition.name(),
        }
    }

    pub fn type_(&self) -> Type {
        match self {
            Self::FunctionDefinition(function_definition) => {
                function_definition.type_().clone().into()
            }
            Self::ValueDefinition(value_definition) => value_definition.type_().clone().into(),
        }
    }

    pub fn to_function_definition(&self) -> Option<&FunctionDefinition> {
        match self {
            Self::FunctionDefinition(function_definition) => Some(function_definition),
            Self::ValueDefinition(_) => None,
        }
    }

    pub fn to_value_definition(&self) -> Option<&ValueDefinition> {
        match self {
            Self::FunctionDefinition(_) => None,
            Self::ValueDefinition(value_definition) => Some(value_definition),
        }
    }

    pub(crate) fn find_variables(&self) -> HashSet<String> {
        match self {
            Self::FunctionDefinition(function_definition) => function_definition.find_variables(),
            Self::ValueDefinition(value_definition) => value_definition.find_variables(),
        }
    }

    pub(crate) fn infer_environment(&self, variables: &HashMap<String, Type>) -> Self {
        match self {
            Self::FunctionDefinition(function_definition) => {
                function_definition.infer_environment(variables).into()
            }
            Self::ValueDefinition(value_definition) => {
                value_definition.infer_environment(variables).into()
            }
        }
    }

    pub(crate) fn convert_types(&self, convert: &impl Fn(&Type) -> Type) -> Self {
        match self {
            Self::FunctionDefinition(function_definition) => {
                function_definition.convert_types(convert).into()
            }
            Self::ValueDefinition(value_definition) => {
                value_definition.convert_types(convert).into()
            }
        }
    }
}

impl From<FunctionDefinition> for Definition {
    fn from(function_definition: FunctionDefinition) -> Self {
        Definition::FunctionDefinition(function_definition)
    }
}

impl From<ValueDefinition> for Definition {
    fn from(function_definition: ValueDefinition) -> Self {
        Definition::ValueDefinition(function_definition)
    }
}
