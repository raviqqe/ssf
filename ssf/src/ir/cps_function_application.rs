use super::continuation::Continuation;
use super::expression::Expression;
use crate::types::Type;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct CpsFunctionApplication {
    function: Arc<Expression>,
    arguments: Vec<Expression>,
    continuation: Arc<Continuation>,
}

impl CpsFunctionApplication {
    pub fn new(
        function: impl Into<Expression>,
        arguments: Vec<Expression>,
        continuation: impl Into<Continuation>,
    ) -> Self {
        Self {
            function: function.into().into(),
            arguments,
            continuation: continuation.into().into(),
        }
    }

    pub fn function(&self) -> &Expression {
        &self.function
    }

    pub fn arguments(&self) -> impl IntoIterator<Item = &Expression> {
        &self.arguments
    }

    pub fn continuation(&self) -> &Continuation {
        &self.continuation
    }

    pub(crate) fn find_variables(&self) -> HashSet<String> {
        self.function
            .find_variables()
            .into_iter()
            .chain(
                self.arguments
                    .iter()
                    .map(|argument| argument.find_variables())
                    .flatten(),
            )
            .chain(self.continuation.find_variables())
            .collect()
    }

    pub(crate) fn infer_environment(&self, variables: &HashMap<String, Type>) -> Self {
        Self {
            function: self.function.infer_environment(variables).into(),
            arguments: self
                .arguments
                .iter()
                .map(|argument| argument.infer_environment(variables))
                .collect(),
            continuation: self.continuation.infer_environment(variables).into(),
        }
    }

    pub(crate) fn convert_types(&self, convert: &impl Fn(&Type) -> Type) -> Self {
        Self {
            function: self.function.convert_types(convert).into(),
            arguments: self
                .arguments
                .iter()
                .map(|argument| argument.convert_types(convert))
                .collect(),
            continuation: self.continuation.convert_types(convert).into(),
        }
    }
}
