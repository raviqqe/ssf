use super::algebraic_case::AlgebraicCase;
use super::primitive_case::PrimitiveCase;
use crate::types::Type;
use std::collections::{HashMap, HashSet};

/// Case expressions match types of algebraic data types with their
/// constructors deconstructing them.
///
/// Their alternatives do not have to be exhaustive. See also options of each
/// compiler for behavior on match failures.
#[derive(Clone, Debug, PartialEq)]
pub enum Case {
    Algebraic(AlgebraicCase),
    Primitive(PrimitiveCase),
}

impl Case {
    pub(crate) fn rename_variables(&self, names: &HashMap<String, String>) -> Self {
        match self {
            Self::Algebraic(algebraic_case) => algebraic_case.rename_variables(names).into(),
            Self::Primitive(primitive_case) => primitive_case.rename_variables(names).into(),
        }
    }

    pub(crate) fn find_variables(&self, excluded_variables: &HashSet<String>) -> HashSet<String> {
        match self {
            Self::Algebraic(algebraic_case) => algebraic_case.find_variables(excluded_variables),
            Self::Primitive(primitive_case) => primitive_case.find_variables(excluded_variables),
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
            Self::Primitive(primitive_case) => primitive_case
                .infer_environment(variables, global_variables)
                .into(),
        }
    }

    pub(crate) fn convert_types(&self, convert: &impl Fn(&Type) -> Type) -> Self {
        match self {
            Self::Algebraic(algebraic_case) => algebraic_case.convert_types(convert).into(),
            Self::Primitive(primitive_case) => primitive_case.convert_types(convert).into(),
        }
    }
}

impl From<AlgebraicCase> for Case {
    fn from(algebraic_case: AlgebraicCase) -> Self {
        Self::Algebraic(algebraic_case)
    }
}

impl From<PrimitiveCase> for Case {
    fn from(primitive_case: PrimitiveCase) -> Self {
        Self::Primitive(primitive_case)
    }
}
