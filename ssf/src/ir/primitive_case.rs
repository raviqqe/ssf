use super::default_alternative::DefaultAlternative;
use super::expression::Expression;
use super::primitive_alternative::PrimitiveAlternative;
use crate::types::{self, Type};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct PrimitiveCase {
    type_: types::Primitive,
    argument: Arc<Expression>,
    alternatives: Vec<PrimitiveAlternative>,
    default_alternative: Option<DefaultAlternative>,
}

impl PrimitiveCase {
    pub fn new(
        type_: types::Primitive,
        argument: impl Into<Expression>,
        alternatives: Vec<PrimitiveAlternative>,
        default_alternative: Option<DefaultAlternative>,
    ) -> Self {
        Self {
            type_,
            argument: Arc::new(argument.into()),
            alternatives,
            default_alternative,
        }
    }

    pub fn type_(&self) -> types::Primitive {
        self.type_
    }

    pub fn argument(&self) -> &Expression {
        &self.argument
    }

    pub fn alternatives(&self) -> &[PrimitiveAlternative] {
        &self.alternatives
    }

    pub fn default_alternative(&self) -> Option<&DefaultAlternative> {
        self.default_alternative.as_ref()
    }

    pub(crate) fn find_variables(&self) -> HashSet<String> {
        let mut variables = self.argument.find_variables();

        for alternative in &self.alternatives {
            variables.extend(alternative.find_variables());
        }

        if let Some(default_alternative) = &self.default_alternative {
            variables.extend(default_alternative.find_variables());
        }

        variables
    }

    pub(crate) fn infer_environment(&self, variables: &HashMap<String, Type>) -> Self {
        Self {
            type_: self.type_,
            argument: self.argument.infer_environment(variables).into(),
            alternatives: self
                .alternatives
                .iter()
                .map(|alternative| alternative.infer_environment(variables))
                .collect(),
            default_alternative: self
                .default_alternative
                .as_ref()
                .map(|default_alternative| {
                    default_alternative.infer_environment(self.type_, variables)
                }),
        }
    }

    pub(crate) fn convert_types(&self, convert: &impl Fn(&Type) -> Type) -> Self {
        Self {
            type_: convert(&self.type_.clone().into())
                .into_primitive()
                .unwrap(),
            argument: self.argument.convert_types(convert).into(),
            alternatives: self
                .alternatives
                .iter()
                .map(|alternative| alternative.convert_types(convert))
                .collect(),
            default_alternative: self
                .default_alternative
                .as_ref()
                .map(|default_alternative| default_alternative.convert_types(convert)),
        }
    }
}
