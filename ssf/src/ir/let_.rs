use super::expression::Expression;
use crate::types::Type;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct Let {
    name: String,
    type_: Type,
    bound_expression: Arc<Expression>,
    expression: Arc<Expression>,
}

impl Let {
    pub fn new(
        name: impl Into<String>,
        type_: impl Into<Type>,
        bound_expression: impl Into<Expression>,
        expression: impl Into<Expression>,
    ) -> Self {
        Self {
            name: name.into(),
            type_: type_.into(),
            bound_expression: bound_expression.into().into(),
            expression: expression.into().into(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn type_(&self) -> &Type {
        &self.type_
    }

    pub fn bound_expression(&self) -> &Expression {
        &self.bound_expression
    }

    pub fn expression(&self) -> &Expression {
        &self.expression
    }

    pub(crate) fn find_variables(&self) -> HashSet<String> {
        self.bound_expression
            .find_variables()
            .into_iter()
            .chain({
                let mut variables = self.expression.find_variables();
                variables.remove(&self.name);
                variables
            })
            .collect()
    }

    pub(crate) fn infer_environment(&self, variables: &HashMap<String, Type>) -> Self {
        Self::new(
            self.name.clone(),
            self.type_.clone(),
            self.bound_expression.infer_environment(&variables),
            {
                let mut variables = variables.clone();

                variables.insert(self.name.clone(), self.type_.clone());

                self.expression.infer_environment(&variables)
            },
        )
    }

    pub(crate) fn convert_types(&self, convert: &impl Fn(&Type) -> Type) -> Self {
        Self::new(
            self.name.clone(),
            convert(&self.type_),
            self.bound_expression.convert_types(convert),
            self.expression.convert_types(convert),
        )
    }
}
