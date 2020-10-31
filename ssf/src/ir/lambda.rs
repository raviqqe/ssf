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
    arguments: Vec<Argument>,
    body: Arc<Expression>,
    result_type: Type,
    type_: types::Function,
}

impl Lambda {
    pub fn new(
        arguments: Vec<Argument>,
        body: impl Into<Expression>,
        result_type: impl Into<Type> + Clone,
    ) -> Self {
        Self::with_environment(vec![], arguments, body, result_type)
    }

    pub(crate) fn with_environment(
        environment: Vec<Argument>,
        arguments: Vec<Argument>,
        body: impl Into<Expression>,
        result_type: impl Into<Type> + Clone,
    ) -> Self {
        Self {
            type_: if arguments.is_empty() {
                todo!("thunk not implemented")
            } else {
                types::canonicalize(
                    &arguments.iter().rev().take(1).fold(
                        types::Function::new(
                            arguments.iter().last().unwrap().type_().clone(),
                            result_type.clone(),
                        )
                        .into(),
                        |function, argument| {
                            types::Function::new(
                                argument.type_().clone(),
                                result_type.clone().into(),
                            )
                            .into()
                        },
                    ),
                )
                .into_function()
                .unwrap()
            },
            environment,
            arguments,
            body: body.into().into(),
            result_type: result_type.into(),
        }
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

    pub(crate) fn rename_variables(&self, names: &HashMap<String, String>) -> Self {
        for argument in &self.arguments {
            names.remove(argument.name());
        }

        Self::with_environment(
            self.environment.clone(),
            self.arguments.clone(),
            self.body.rename_variables(&names),
            self.result_type.clone(),
        )
    }

    pub(crate) fn find_variables(&self, excluded_variables: &HashSet<String>) -> HashSet<String> {
        let mut excluded_variables = excluded_variables.clone();

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
        let mut excluded_variables = global_variables.clone();

        for argument in &self.arguments {
            variables.insert(argument.name().into(), argument.type_().clone());
            excluded_variables.insert(argument.name().into());
        }

        Self::with_environment(
            self.body
                .find_variables(&excluded_variables)
                .iter()
                .map(|name| Argument::new(name, original_variables[name].clone()))
                .collect(),
            self.arguments.clone(),
            self.body.infer_environment(&variables, global_variables),
            self.result_type.clone(),
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
            .infer_environment(&Default::default(), &Default::default()),
            Lambda::with_environment(
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
            Lambda::new(
                vec![Argument::new("x", types::Primitive::Float64)],
                Variable::new("y"),
                types::Primitive::Float64
            )
            .infer_environment(
                &vec![("y".into(), types::Primitive::Float64.into())]
                    .drain(..)
                    .collect(),
                &Default::default()
            ),
            Lambda::with_environment(
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
            Lambda::new(
                vec![Argument::new("x", types::Primitive::Float64)],
                Variable::new("y"),
                types::Primitive::Float64
            )
            .infer_environment(&variables, &Default::default())
            .infer_environment(&variables, &Default::default()),
            Lambda::with_environment(
                "f",
                vec![Argument::new("y", types::Primitive::Float64)],
                vec![Argument::new("x", types::Primitive::Float64)],
                Variable::new("y"),
                types::Primitive::Float64
            )
        );
    }
}
