use super::algebraic_case::AlgebraicCase;
use crate::types::Type;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq)]
pub enum Case {
    Algebraic(AlgebraicCase),
}

impl Case {
    pub(crate) fn rename_variables(&self, names: &HashMap<String, String>) -> Self {
        match self {
            Self::Algebraic(algebraic_case) => algebraic_case.rename_variables(names).into(),
        }
    }

    pub(crate) fn find_variables(&self, excluded_variables: &HashSet<String>) -> HashSet<String> {
        match self {
            Self::Algebraic(algebraic_case) => algebraic_case.find_variables(excluded_variables),
        }
    }

    pub(crate) fn infer_environment(
        &self,
        variables: &HashMap<String, Type>,
        global_variables: &HashSet<String>,
    ) -> Self {
        match self {
            Self::Algebraic(algebraic_case) => algebraic_case
                .infer_environment(variables, global_variables)
                .into(),
        }
    }

    pub(crate) fn convert_types(&self, convert: &impl Fn(&Type) -> Type) -> Self {
        match self {
            Self::Algebraic(algebraic_case) => algebraic_case.convert_types(convert).into(),
        }
    }
}

impl From<AlgebraicCase> for Case {
    fn from(algebraic_case: AlgebraicCase) -> Self {
        Self::Algebraic(algebraic_case)
    }
}
