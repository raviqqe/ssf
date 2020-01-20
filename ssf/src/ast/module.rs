use super::declaration::Declaration;
use super::definition::Definition;
use crate::analysis::{check_types, sort_global_variables, AnalysisError, TypeCheckError};

#[derive(Clone, Debug, PartialEq)]
pub struct Module {
    declarations: Vec<Declaration>,
    definitions: Vec<Definition>,
}

impl Module {
    pub fn new(declarations: Vec<Declaration>, definitions: Vec<Definition>) -> Self {
        Self {
            declarations,
            definitions,
        }
    }

    pub fn declarations(&self) -> &[Declaration] {
        &self.declarations
    }

    pub fn definitions(&self) -> &[Definition] {
        &self.definitions
    }

    pub fn check_types(&self) -> Result<(), TypeCheckError> {
        check_types(self)
    }

    pub fn sort_global_variables(&self) -> Result<Vec<&str>, AnalysisError> {
        sort_global_variables(self)
    }
}
