use super::expression::Expression;
use super::Argument;
use crate::types::{self, Type};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionDefinition {
    name: String,
    environment: Vec<Argument>,
    arguments: Vec<Argument>,
    body: Expression,
    result_type: types::Value,
    type_: types::Function,
}

impl FunctionDefinition {
    pub fn new(
        name: impl Into<String>,
        arguments: Vec<Argument>,
        body: impl Into<Expression>,
        result_type: types::Value,
    ) -> Self {
        Self {
            type_: types::Function::new(
                arguments
                    .iter()
                    .map(|argument| argument.type_().clone())
                    .collect(),
                result_type.clone(),
            ),
            name: name.into(),
            environment: vec![],
            arguments,
            body: body.into(),
            result_type,
        }
    }

    pub(crate) fn with_environment(
        name: impl Into<String>,
        environment: Vec<Argument>,
        arguments: Vec<Argument>,
        body: impl Into<Expression>,
        result_type: types::Value,
    ) -> Self {
        Self {
            type_: types::Function::new(
                arguments
                    .iter()
                    .map(|argument| argument.type_().clone())
                    .collect(),
                result_type.clone(),
            ),
            name: name.into(),
            environment,
            arguments,
            body: body.into(),
            result_type,
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

    pub fn result_type(&self) -> &types::Value {
        &self.result_type
    }

    pub fn type_(&self) -> &types::Function {
        &self.type_
    }

    pub(crate) fn rename_variables(&self, names: &HashMap<String, String>) -> Self {
        let mut names = names.clone();

        names.remove(self.name.as_str());

        for argument in &self.arguments {
            names.remove(argument.name());
        }

        Self::with_environment(
            self.name.clone(),
            self.environment.clone(),
            self.arguments.clone(),
            self.body.rename_variables(&names),
            self.result_type.clone(),
        )
    }

    pub(crate) fn find_variables(&self, excluded_variables: &HashSet<String>) -> HashSet<String> {
        let mut excluded_variables = excluded_variables.clone();

        excluded_variables.insert(self.name.clone());

        excluded_variables.extend(
            self.arguments
                .iter()
                .map(|argument| argument.name().into())
                .collect::<HashSet<_>>(),
        );

        self.body.find_variables(&excluded_variables)
    }

    pub(crate) fn infer_environment(
        &self,
        original_variables: &HashMap<String, Type>,
        global_variables: &HashSet<String>,
    ) -> Self {
        let mut variables = original_variables.clone();

        for argument in &self.arguments {
            variables.insert(argument.name().into(), argument.type_().clone());
        }

        Self::with_environment(
            self.name.clone(),
            self.find_variables(global_variables)
                .iter()
                .filter(|name| {
                    self.arguments
                        .iter()
                        .all(|argument| argument.name() != name.as_str())
                })
                .map(|name| Argument::new(name, original_variables[name].clone()))
                .collect(),
            self.arguments.clone(),
            self.body.infer_environment(&variables, global_variables),
            self.result_type.clone(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::*;

    #[test]
    fn infer_empty_environment() {
        assert_eq!(
            FunctionDefinition::new(
                "f",
                vec![Argument::new("x", types::Value::Number)],
                Expression::Number(42.0),
                types::Value::Number
            )
            .infer_environment(&Default::default(), &Default::default()),
            FunctionDefinition::with_environment(
                "f",
                vec![],
                vec![Argument::new("x", types::Value::Number)],
                Expression::Number(42.0),
                types::Value::Number
            )
        );
    }

    #[test]
    fn infer_environment() {
        assert_eq!(
            FunctionDefinition::new(
                "f",
                vec![Argument::new("x", types::Value::Number)],
                Variable::new("y"),
                types::Value::Number
            )
            .infer_environment(
                &vec![("y".into(), types::Value::Number.into())]
                    .drain(..)
                    .collect(),
                &Default::default()
            ),
            FunctionDefinition::with_environment(
                "f",
                vec![Argument::new("y", types::Value::Number)],
                vec![Argument::new("x", types::Value::Number)],
                Variable::new("y"),
                types::Value::Number
            )
        );
    }

    #[test]
    fn infer_environment_idempotently() {
        let variables = vec![("y".into(), types::Value::Number.into())]
            .drain(..)
            .collect();

        assert_eq!(
            FunctionDefinition::new(
                "f",
                vec![Argument::new("x", types::Value::Number)],
                Variable::new("y"),
                types::Value::Number
            )
            .infer_environment(&variables, &Default::default())
            .infer_environment(&variables, &Default::default()),
            FunctionDefinition::with_environment(
                "f",
                vec![Argument::new("y", types::Value::Number)],
                vec![Argument::new("x", types::Value::Number)],
                Variable::new("y"),
                types::Value::Number
            )
        );
    }
}
