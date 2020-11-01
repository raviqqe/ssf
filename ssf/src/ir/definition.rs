use super::expression::Expression;
use crate::types::Type;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq)]
pub struct Definition {
    name: String,
    body: Expression,
    type_: Type,
}

impl Definition {
    pub fn new(
        name: impl Into<String>,
        body: impl Into<Expression>,
        type_: impl Into<Type>,
    ) -> Self {
        Self {
            name: name.into(),
            body: body.into(),
            type_: type_.into(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn body(&self) -> &Expression {
        &self.body
    }

    pub fn type_(&self) -> &Type {
        &self.type_
    }

    pub(crate) fn rename_variables(&self, names: &HashMap<String, String>) -> Self {
        Self::new(
            self.name.clone(),
            self.body.rename_variables(names),
            self.type_.clone(),
        )
    }

    pub(crate) fn find_free_variables(&self) -> HashSet<String> {
        self.body.find_free_variables()
    }

    pub(crate) fn infer_environment(&self, variables: &HashMap<String, Type>) -> Self {
        Self::new(
            self.name.clone(),
            self.body.infer_environment(variables),
            self.type_.clone(),
        )
    }

    pub(crate) fn convert_types(&self, convert: &impl Fn(&Type) -> Type) -> Self {
        Self::new(
            self.name.clone(),
            self.body.convert_types(convert),
            convert(&self.type_.clone().into()),
        )
    }
}
