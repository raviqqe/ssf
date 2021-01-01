use super::argument::Argument;
use super::expression::Expression;
use crate::types::Type;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq)]
pub struct LocalContinuation {
    // Environment is inferred on module creation and this field is used just
    // as its cache.  So it must be safe to clone definitions inside a
    // module and use it on creation of another module.
    environment: Vec<Argument>,
    argument: Argument,
    body: Expression,
}

impl LocalContinuation {
    pub fn new(argument: Argument, body: impl Into<Expression>) -> Self {
        Self::with_environment(vec![], argument, body)
    }

    pub fn with_environment(
        environment: Vec<Argument>,
        argument: Argument,
        body: impl Into<Expression>,
    ) -> Self {
        Self {
            environment,
            argument,
            body: body.into(),
        }
    }

    pub fn environment(&self) -> &[Argument] {
        &self.environment
    }

    pub fn argument(&self) -> &Argument {
        &self.argument
    }

    pub fn body(&self) -> &Expression {
        &self.body
    }

    pub(crate) fn find_variables(&self) -> HashSet<String> {
        let mut variables = self.body.find_variables();

        variables.remove(self.argument.name());

        variables
    }

    pub(crate) fn infer_environment(&self, variables: &HashMap<String, Type>) -> Self {
        Self::with_environment(
            self.body
                .find_variables()
                .iter()
                .filter_map(|name| {
                    variables
                        .get(name)
                        .map(|type_| Argument::new(name, type_.clone()))
                })
                .collect(),
            self.argument.clone(),
            self.body.infer_environment(
                &variables
                    .into_iter()
                    .map(|(name, type_)| (name.clone(), type_.clone()))
                    .chain(vec![(
                        self.argument.name().into(),
                        self.argument.type_().clone(),
                    )])
                    .collect(),
            ),
        )
    }

    pub(crate) fn convert_types(&self, convert: &impl Fn(&Type) -> Type) -> Self {
        Self {
            environment: self
                .environment
                .iter()
                .map(|argument| argument.convert_types(convert))
                .collect(),
            argument: self.argument.convert_types(convert),
            body: self.body.convert_types(convert),
        }
    }
}
