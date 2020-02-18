use super::constructor::Constructor;
use super::expression::Expression;
use crate::types::Type;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq)]
pub struct AlgebraicAlternative {
    constructor: Constructor,
    element_names: Vec<String>,
    expression: Expression,
}

impl AlgebraicAlternative {
    pub fn new(
        constructor: Constructor,
        element_names: Vec<String>,
        expression: impl Into<Expression>,
    ) -> Self {
        Self {
            constructor,
            element_names,
            expression: expression.into(),
        }
    }

    pub fn constructor(&self) -> &Constructor {
        &self.constructor
    }

    pub fn elements(&self) -> impl IntoIterator<Item = (&str, &Type)> {
        self.element_names
            .iter()
            .map(|name| name.as_str())
            .zip(self.constructor.element_types())
    }

    pub fn expression(&self) -> &Expression {
        &self.expression
    }

    pub(crate) fn rename_variables(&self, names: &HashMap<String, String>) -> Self {
        let mut names = names.clone();

        for element_name in &self.element_names {
            names.remove(element_name);
        }

        Self {
            constructor: self.constructor.clone(),
            element_names: self.element_names.clone(),
            expression: self.expression.rename_variables(&names),
        }
    }

    pub(crate) fn find_variables(&self, excluded_variables: &HashSet<String>) -> HashSet<String> {
        let mut excluded_variables = excluded_variables.clone();

        for element_name in &self.element_names {
            excluded_variables.insert(element_name.into());
        }

        self.expression.find_variables(&excluded_variables)
    }

    pub(crate) fn infer_environment(
        &self,
        variables: &HashMap<String, Type>,
        global_variables: &HashSet<String>,
    ) -> Self {
        Self {
            constructor: self.constructor.clone(),
            element_names: self.element_names.clone(),
            expression: self
                .expression
                .infer_environment(variables, global_variables),
        }
    }

    pub(crate) fn convert_types(&self, convert: &impl Fn(&Type) -> Type) -> Self {
        Self {
            constructor: self.constructor.convert_types(convert),
            element_names: self.element_names.clone(),
            expression: self.expression.convert_types(convert),
        }
    }
}
