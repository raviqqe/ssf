use super::argument::Argument;
use super::expression::Expression;
use crate::types::{self, Type};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq)]
pub struct Definition {
    name: String,
    // Environment is inferred on module creation and this field is used just
    // as its cache.  So it must be safe to clone definitions inside a
    // module and use it on creation of another module.
    environment: Vec<Argument>,
    arguments: Vec<Argument>,
    body: Expression,
    result_type: Type,
    type_: types::Function,
    is_thunk: bool,
}

impl Definition {
    pub fn new(
        name: impl Into<String>,
        arguments: Vec<Argument>,
        body: impl Into<Expression>,
        result_type: impl Into<Type> + Clone,
    ) -> Self {
        Self::with_options(name, vec![], arguments, body, result_type, false)
    }

    pub fn thunk(
        name: impl Into<String>,
        arguments: Vec<Argument>,
        body: impl Into<Expression>,
        result_type: impl Into<Type> + Clone,
    ) -> Self {
        Self::with_options(name, vec![], arguments, body, result_type, true)
    }

    #[cfg(test)]
    pub(crate) fn with_environment(
        name: impl Into<String>,
        environment: Vec<Argument>,
        arguments: Vec<Argument>,
        body: impl Into<Expression>,
        result_type: impl Into<Type> + Clone,
    ) -> Self {
        Self::with_options(name, environment, arguments, body, result_type, false)
    }

    pub(crate) fn with_options(
        name: impl Into<String>,
        environment: Vec<Argument>,
        arguments: Vec<Argument>,
        body: impl Into<Expression>,
        result_type: impl Into<Type> + Clone,
        is_thunk: bool,
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
            name: name.into(),
            environment,
            arguments,
            body: body.into(),
            result_type: result_type.into(),
            is_thunk,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
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

    pub fn type_(&self) -> &types::Function {
        &self.type_
    }

    pub fn is_thunk(&self) -> bool {
        self.is_thunk
    }

    pub(crate) fn rename_variables(&self, names: &HashMap<String, String>) -> Self {
        let mut names = names.clone();

        names.remove(self.name.as_str());

        for argument in &self.arguments {
            names.remove(argument.name());
        }

        Self::with_options(
            self.name.clone(),
            self.environment.clone(),
            self.arguments.clone(),
            self.body.rename_variables(&names),
            self.result_type.clone(),
            self.is_thunk,
        )
    }

    pub(crate) fn find_variables(&self) -> HashSet<String> {
        let mut variables = self.body.find_variables();

        variables.remove(&self.name);

        for argument in &self.arguments {
            variables.remove(argument.name());
        }

        variables
    }

    pub(crate) fn infer_environment(&self, variables: &HashMap<String, Type>) -> Self {
        // Do not include this function itself in variables as it can be global.

        Self::with_options(
            self.name.clone(),
            self.body
                .find_variables()
                .iter()
                .filter_map(|name| {
                    variables
                        .get(name)
                        .map(|type_| Argument::new(name, type_.clone()))
                })
                .collect(),
            self.arguments.clone(),
            {
                let mut variables = variables.clone();

                for argument in &self.arguments {
                    variables.insert(argument.name().into(), argument.type_().clone());
                }

                self.body.infer_environment(&variables)
            },
            self.result_type.clone(),
            self.is_thunk,
        )
    }

    pub(crate) fn convert_types(&self, convert: &impl Fn(&Type) -> Type) -> Self {
        Self {
            name: self.name.clone(),
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
            body: self.body.convert_types(convert),
            result_type: convert(&self.result_type.clone()),
            type_: convert(&self.type_.clone().into()).into_function().unwrap(),
            is_thunk: self.is_thunk,
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
            Definition::new(
                "f",
                vec![Argument::new("x", types::Primitive::Float64)],
                42.0,
                types::Primitive::Float64
            )
            .infer_environment(&Default::default()),
            Definition::with_environment(
                "f",
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
            Definition::new(
                "f",
                vec![Argument::new("x", types::Primitive::Float64)],
                Variable::new("y"),
                types::Primitive::Float64
            )
            .infer_environment(
                &vec![("y".into(), types::Primitive::Float64.into())]
                    .drain(..)
                    .collect()
            ),
            Definition::with_environment(
                "f",
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
            Definition::new(
                "f",
                vec![Argument::new("x", types::Primitive::Float64)],
                Variable::new("y"),
                types::Primitive::Float64
            )
            .infer_environment(&variables)
            .infer_environment(&variables),
            Definition::with_environment(
                "f",
                vec![Argument::new("y", types::Primitive::Float64)],
                vec![Argument::new("x", types::Primitive::Float64)],
                Variable::new("y"),
                types::Primitive::Float64
            )
        );
    }
}
