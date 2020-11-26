use super::expression::Expression;
use super::primitive_alternative::PrimitiveAlternative;
use crate::types::Type;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct PrimitiveCase {
    argument: Arc<Expression>,
    alternatives: Vec<PrimitiveAlternative>,
    default_alternative: Option<Arc<Expression>>,
}

impl PrimitiveCase {
    pub fn new(
        argument: impl Into<Expression>,
        alternatives: Vec<PrimitiveAlternative>,
        default_alternative: Option<Expression>,
    ) -> Self {
        Self {
            argument: Arc::new(argument.into()),
            alternatives,
            default_alternative: default_alternative.map(|expression| expression.into()),
        }
    }

    pub fn argument(&self) -> &Expression {
        &self.argument
    }

    pub fn alternatives(&self) -> &[PrimitiveAlternative] {
        &self.alternatives
    }

    pub fn default_alternative(&self) -> Option<&Expression> {
        self.default_alternative
            .as_ref()
            .map(|expression| expression.as_ref())
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
            argument: self.argument.infer_environment(variables).into(),
            alternatives: self
                .alternatives
                .iter()
                .map(|alternative| alternative.infer_environment(variables))
                .collect(),
            default_alternative: self
                .default_alternative
                .as_ref()
                .map(|expression| expression.infer_environment(variables).into()),
        }
    }

    pub(crate) fn convert_types(&self, convert: &impl Fn(&Type) -> Type) -> Self {
        Self {
            argument: self.argument.convert_types(convert).into(),
            alternatives: self
                .alternatives
                .iter()
                .map(|alternative| alternative.convert_types(convert))
                .collect(),
            default_alternative: self
                .default_alternative
                .as_ref()
                .map(|expression| expression.convert_types(convert).into()),
        }
    }
}
