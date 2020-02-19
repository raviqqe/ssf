use super::algebraic_alternative::AlgebraicAlternative;
use super::default_alternative::DefaultAlternative;
use super::expression::Expression;
use crate::types::Type;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq)]
pub struct AlgebraicCase {
    argument: Box<Expression>,
    alternatives: Vec<AlgebraicAlternative>,
    default_alternative: Option<DefaultAlternative>,
}

impl AlgebraicCase {
    pub fn new(
        argument: impl Into<Expression>,
        alternatives: Vec<AlgebraicAlternative>,
        default_alternative: Option<DefaultAlternative>,
    ) -> Self {
        Self {
            argument: Box::new(argument.into()),
            alternatives,
            default_alternative,
        }
    }

    pub fn argument(&self) -> &Expression {
        &self.argument
    }

    pub fn alternatives(&self) -> &[AlgebraicAlternative] {
        &self.alternatives
    }

    pub fn default_alternative(&self) -> Option<&DefaultAlternative> {
        self.default_alternative.as_ref()
    }

    pub(crate) fn rename_variables(&self, names: &HashMap<String, String>) -> Self {
        Self {
            argument: self.argument.rename_variables(names).into(),
            alternatives: self
                .alternatives
                .iter()
                .map(|alternative| alternative.rename_variables(names))
                .collect(),
            default_alternative: self
                .default_alternative
                .as_ref()
                .map(|default_alternative| default_alternative.rename_variables(names)),
        }
    }

    pub(crate) fn find_variables(&self, excluded_variables: &HashSet<String>) -> HashSet<String> {
        let mut variables = self.argument.find_variables(excluded_variables);

        for alternative in &self.alternatives {
            variables.extend(alternative.find_variables(excluded_variables));
        }

        if let Some(default_alternative) = &self.default_alternative {
            variables.extend(default_alternative.find_variables(excluded_variables));
        }

        variables
    }

    pub(crate) fn infer_environment(
        &self,
        variables: &HashMap<String, Type>,
        global_variables: &HashSet<String>,
    ) -> Self {
        Self {
            argument: self
                .argument
                .infer_environment(variables, global_variables)
                .into(),
            alternatives: self
                .alternatives
                .iter()
                .map(|alternative| alternative.infer_environment(variables, global_variables))
                .collect(),
            default_alternative: self
                .default_alternative
                .as_ref()
                .map(|default_alternative| {
                    default_alternative.infer_environment(variables, global_variables)
                }),
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
                .map(|default_alternative| default_alternative.convert_types(convert)),
        }
    }
}
