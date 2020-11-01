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

    pub fn element_names(&self) -> &[String] {
        &self.element_names
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

    pub(crate) fn find_free_variables(&self, initialized: bool) -> HashSet<String> {
        let mut variables = self.expression.find_free_variables(initialized);

        for element_name in &self.element_names {
            variables.remove(element_name);
        }

        variables
    }

    pub(crate) fn infer_environment(&self, variables: &HashMap<String, Type>) -> Self {
        let mut variables = variables.clone();

        for (name, type_) in self
            .element_names
            .iter()
            .zip(self.constructor.constructor_type().elements())
        {
            variables.insert(name.into(), type_.clone());
        }

        Self {
            constructor: self.constructor.clone(),
            element_names: self.element_names.clone(),
            expression: self.expression.infer_environment(&variables),
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
