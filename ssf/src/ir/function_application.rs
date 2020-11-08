use super::expression::Expression;
use crate::types::Type;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionApplication {
    function: Arc<Expression>,
    argument: Arc<Expression>,
}

impl FunctionApplication {
    pub fn new(function: impl Into<Expression>, argument: impl Into<Expression>) -> Self {
        Self {
            function: function.into().into(),
            argument: argument.into().into(),
        }
    }

    pub fn function(&self) -> &Expression {
        &self.function
    }

    pub fn argument(&self) -> &Expression {
        &self.argument
    }

    pub fn first_function(&self) -> &Expression {
        let mut function: &Expression = &self.function;

        while let Expression::FunctionApplication(function_application) = function {
            function = function_application.function();
        }

        function
    }

    pub fn arguments(&self) -> impl IntoIterator<Item = &Expression> {
        let mut arguments = vec![self.argument()];
        let mut expression = self;

        while let Expression::FunctionApplication(function_application) = expression.function() {
            arguments.push(function_application.argument());
            expression = function_application;
        }

        arguments.reverse();

        arguments
    }

    pub(crate) fn rename_variables(&self, names: &HashMap<String, String>) -> Self {
        Self::new(
            self.function.rename_variables(names),
            self.argument.rename_variables(names),
        )
    }

    pub(crate) fn find_variables(&self) -> HashSet<String> {
        self.function
            .find_variables()
            .into_iter()
            .chain(self.argument.find_variables())
            .collect()
    }

    pub(crate) fn infer_environment(&self, variables: &HashMap<String, Type>) -> Self {
        Self::new(
            self.function.infer_environment(variables),
            self.argument.infer_environment(variables),
        )
    }

    pub(crate) fn convert_types(&self, convert: &impl Fn(&Type) -> Type) -> Self {
        Self::new(
            self.function.convert_types(convert),
            self.argument.convert_types(convert),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::super::variable::Variable;
    use super::*;

    #[test]
    fn first_function() {
        assert_eq!(
            FunctionApplication::new(Variable::new("f"), 42.0).first_function(),
            &Variable::new("f").into()
        );

        assert_eq!(
            FunctionApplication::new(FunctionApplication::new(Variable::new("f"), 1.0), 2.0)
                .first_function(),
            &Variable::new("f").into()
        );
    }

    #[test]
    fn arguments() {
        assert_eq!(
            FunctionApplication::new(Variable::new("f"), 42.0)
                .arguments()
                .into_iter()
                .cloned()
                .collect::<Vec<_>>(),
            vec![42.0.into()]
        );

        assert_eq!(
            FunctionApplication::new(FunctionApplication::new(Variable::new("f"), 1.0), 2.0)
                .arguments()
                .into_iter()
                .cloned()
                .collect::<Vec<_>>(),
            vec![1.0.into(), 2.0.into()]
        );
    }
}
