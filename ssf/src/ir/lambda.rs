use super::expression::Expression;
use super::Argument;
use crate::types::{self, Type};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct Lambda {
    // Environment is inferred on module creation and this field is used just
    // as its cache.  So it must be safe to clone function definitions inside a
    // module and use it on creation of another module.
    environment: Vec<Argument>,
    // Arguments should not be empty.
    arguments: Vec<Argument>,
    body: Arc<Expression>,
    result_type: Type,
    type_: types::Function,
    updatable: bool,
}

impl Lambda {
    pub fn new(
        arguments: Vec<Argument>,
        body: impl Into<Expression>,
        result_type: impl Into<Type> + Clone,
    ) -> Self {
        Self::with_options(vec![], arguments, body, result_type, false)
    }

    pub fn updatable(
        arguments: Vec<Argument>,
        body: impl Into<Expression>,
        result_type: impl Into<Type> + Clone,
    ) -> Self {
        Self::with_options(vec![], arguments, body, result_type, true)
    }

    fn with_options(
        environment: Vec<Argument>,
        arguments: Vec<Argument>,
        body: impl Into<Expression>,
        result_type: impl Into<Type> + Clone,
        updatable: bool,
    ) -> Self {
        Self {
            type_: types::canonicalize(
                &arguments.iter().rev().skip(1).fold(
                    types::Function::new(
                        arguments.iter().last().unwrap().type_().clone(),
                        result_type.clone(),
                    )
                    .into(),
                    |result, argument| {
                        types::Function::new(argument.type_().clone(), result).into()
                    },
                ),
            )
            .into_function()
            .unwrap(),
            environment,
            arguments,
            body: body.into().into(),
            result_type: result_type.into(),
            updatable,
        }
    }

    #[cfg(test)]
    pub(crate) fn with_environment(
        environment: Vec<Argument>,
        arguments: Vec<Argument>,
        body: impl Into<Expression>,
        result_type: impl Into<Type> + Clone,
    ) -> Self {
        Self::with_options(environment, arguments, body, result_type, false)
    }

    pub fn environment(&self) -> &[Argument] {
        &self.environment
    }

    pub fn arguments(&self) -> &[Argument] {
        &self.arguments
    }

    pub fn body(&self) -> &Expression {
        &self.body
    }

    pub fn result_type(&self) -> &Type {
        &self.result_type
    }

    pub fn is_updatable(&self) -> bool {
        self.updatable
    }

    pub fn type_(&self) -> &types::Function {
        &self.type_
    }

    pub(crate) fn rename_variables(&self, names: &HashMap<String, String>) -> Self {
        let mut names = names.clone();

        for argument in &self.arguments {
            names.remove(argument.name());
        }

        Self::with_options(
            self.environment.clone(),
            self.arguments.clone(),
            self.body.rename_variables(&names),
            self.result_type.clone(),
            self.updatable,
        )
    }

    pub(crate) fn find_free_variables(&self, initialized: bool) -> HashSet<String> {
        if initialized {
            Default::default()
        } else {
            let mut variables = self.body.find_free_variables(initialized);

            for argument in &self.arguments {
                variables.remove(argument.name());
            }

            variables
        }
    }

    pub(crate) fn infer_environment(&self, variables: &HashMap<String, Type>) -> Self {
        let environment = self
            .body
            .find_free_variables(false)
            .iter()
            .filter_map(|name| {
                variables
                    .get(name)
                    .map(|type_| Argument::new(name, type_.clone()))
            })
            .collect();

        let mut variables = variables.clone();

        for argument in &self.arguments {
            variables.insert(argument.name().into(), argument.type_().clone());
        }

        Self::with_options(
            environment,
            self.arguments.clone(),
            self.body.infer_environment(&variables),
            self.result_type.clone(),
            self.updatable,
        )
    }

    pub(crate) fn convert_types(&self, convert: &impl Fn(&Type) -> Type) -> Self {
        Self {
            environment: self
                .environment
                .iter()
                .map(|argument| argument.convert_types(convert))
                .collect(),
            arguments: self
                .arguments
                .iter()
                .map(|argument| argument.convert_types(convert))
                .collect(),
            body: self.body.convert_types(convert).into(),
            result_type: convert(&self.result_type.clone().into()),
            type_: convert(&self.type_.clone().into()).into_function().unwrap(),
            updatable: self.updatable,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::*;

    #[test]
    fn infer_empty_environment() {
        assert_eq!(
            Lambda::new(
                vec![Argument::new("x", types::Primitive::Float64)],
                42.0,
                types::Primitive::Float64
            )
            .infer_environment(&Default::default()),
            Lambda::with_environment(
                vec![],
                vec![Argument::new("x", types::Primitive::Float64)],
                42.0,
                types::Primitive::Float64
            )
        );
    }

    #[test]
    fn infer_environment() {
        assert_eq!(
            Lambda::new(
                vec![Argument::new("x", types::Primitive::Float64)],
                Variable::new("y"),
                types::Primitive::Float64
            )
            .infer_environment(
                &vec![("y".into(), types::Primitive::Float64.into())]
                    .drain(..)
                    .collect(),
            ),
            Lambda::with_environment(
                vec![Argument::new("y", types::Primitive::Float64)],
                vec![Argument::new("x", types::Primitive::Float64)],
                Variable::new("y"),
                types::Primitive::Float64
            )
        );
    }

    #[test]
    fn infer_environment_idempotently() {
        let variables = vec![("y".into(), types::Primitive::Float64.into())]
            .drain(..)
            .collect();

        assert_eq!(
            Lambda::new(
                vec![Argument::new("x", types::Primitive::Float64)],
                Variable::new("y"),
                types::Primitive::Float64
            )
            .infer_environment(&variables)
            .infer_environment(&variables),
            Lambda::with_environment(
                vec![Argument::new("y", types::Primitive::Float64)],
                vec![Argument::new("x", types::Primitive::Float64)],
                Variable::new("y"),
                types::Primitive::Float64
            )
        );
    }
}
