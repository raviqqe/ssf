use super::local_continuation::LocalContinuation;
use crate::types::Type;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq)]
pub enum Continuation {
    Current,
    Local(LocalContinuation),
}

impl Continuation {
    pub(crate) fn find_variables(&self) -> HashSet<String> {
        match self {
            Continuation::Current => Default::default(),
            Continuation::Local(continuation) => continuation.find_variables(),
        }
    }

    pub(crate) fn infer_environment(&self, variables: &HashMap<String, Type>) -> Self {
        match self {
            Continuation::Current => self.clone(),
            Continuation::Local(continuation) => continuation.infer_environment(variables).into(),
        }
    }

    pub(crate) fn convert_types(&self, convert: &impl Fn(&Type) -> Type) -> Self {
        match self {
            Continuation::Current => self.clone(),
            Continuation::Local(continuation) => continuation.convert_types(convert).into(),
        }
    }
}

impl From<LocalContinuation> for Continuation {
    fn from(continuation: LocalContinuation) -> Self {
        Self::Local(continuation)
    }
}
