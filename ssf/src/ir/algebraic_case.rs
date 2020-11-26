use super::algebraic_alternative::AlgebraicAlternative;
use super::expression::Expression;
use crate::types::{self, Type};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct AlgebraicCase {
    type_: types::Algebraic,
    argument: Arc<Expression>,
    alternatives: Vec<AlgebraicAlternative>,
    default_alternative: Option<Arc<Expression>>,
}

impl AlgebraicCase {
    pub fn new(
        type_: types::Algebraic,
        argument: impl Into<Expression>,
        alternatives: Vec<AlgebraicAlternative>,
        default_alternative: Option<Expression>,
    ) -> Self {
        Self {
            type_,
            argument: Arc::new(argument.into()),
            alternatives,
            default_alternative: default_alternative.map(|expression| expression.into()),
        }
    }

    pub fn type_(&self) -> &types::Algebraic {
        &self.type_
    }

    pub fn argument(&self) -> &Expression {
        &self.argument
    }

    pub fn alternatives(&self) -> &[AlgebraicAlternative] {
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
            type_: self.type_.clone(),
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
            type_: convert(&self.type_.clone().into())
                .into_algebraic()
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
                .map(|expression| expression.convert_types(convert).into()),
        }
    }
}
