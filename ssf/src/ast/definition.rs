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

    pub(crate) fn find_variables(&self, excluded_variables: &HashSet<String>) -> HashSet<String> {
        match self {
            Self::FunctionDefinition(function_definition) => {
                function_definition.find_variables(excluded_variables)
            }
            Self::ValueDefinition(value_definition) => {
                value_definition.find_variables(excluded_variables)
            }
        }
    }

    pub(crate) fn infer_environment(
        &self,
        variables: &HashMap<String, Type>,
        global_variables: &HashSet<String>,
    ) -> Self {
        match self {
            Self::FunctionDefinition(function_definition) => function_definition
                .infer_environment(variables, global_variables)
                .into(),
            Self::ValueDefinition(value_definition) => value_definition
                .infer_environment(variables, global_variables)
                .into(),
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
