use super::function_definition::*;
use super::value_definition::*;
use std::collections::HashSet;

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
