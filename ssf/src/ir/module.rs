use super::declaration::Declaration;
use super::definition::Definition;
use super::foreign_declaration::ForeignDeclaration;
use crate::types::canonicalize;

#[derive(Clone, Debug, PartialEq)]
pub struct Module {
    foreign_declarations: Vec<ForeignDeclaration>,
    declarations: Vec<Declaration>,
    definitions: Vec<Definition>,
}

impl Module {
    pub fn new(
        foreign_declarations: Vec<ForeignDeclaration>,
        declarations: Vec<Declaration>,
        definitions: Vec<Definition>,
    ) -> Self {
        Self {
            foreign_declarations: foreign_declarations
                .iter()
                .map(|declaration| declaration.convert_types(&canonicalize))
                .collect(),
            declarations: declarations
                .iter()
                .map(|declaration| declaration.convert_types(&canonicalize))
                .collect(),
            definitions: definitions
                .iter()
                .map(|definition| definition.convert_types(&canonicalize))
                .map(|definition| definition.infer_environment(&Default::default()))
                .collect(),
        }
    }

    pub fn foreign_declarations(&self) -> &[ForeignDeclaration] {
        &self.foreign_declarations
    }

    pub fn declarations(&self) -> &[Declaration] {
        &self.declarations
    }

    pub fn definitions(&self) -> &[Definition] {
        &self.definitions
    }
}
