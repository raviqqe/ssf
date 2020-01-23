use super::declaration::Declaration;
use super::definition::Definition;
use super::function_definition::FunctionDefinition;
use super::value_definition::ValueDefinition;
use crate::analysis::{check_types, sort_global_variables, AnalysisError, TypeCheckError};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub struct Module {
    declarations: Vec<Declaration>,
    definitions: Vec<Definition>,
}

impl Module {
    pub fn new(declarations: Vec<Declaration>, definitions: Vec<Definition>) -> Self {
        Self {
            declarations,
            definitions,
        }
    }

    pub fn declarations(&self) -> &[Declaration] {
        &self.declarations
    }

    pub fn definitions(&self) -> &[Definition] {
        &self.definitions
    }

    pub fn check_types(&self) -> Result<(), TypeCheckError> {
        check_types(self)
    }

    pub fn sort_global_variables(&self) -> Result<Vec<&str>, AnalysisError> {
        sort_global_variables(self)
    }

    pub fn rename_global_definitions(&self, names: &HashMap<String, String>) -> Self {
        Self::new(
            self.declarations.to_vec(),
            self.definitions
                .iter()
                .map(|definition| match definition {
                    Definition::FunctionDefinition(function_definition) => FunctionDefinition::new(
                        names
                            .get(function_definition.name())
                            .cloned()
                            .unwrap_or_else(|| function_definition.name().into()),
                        function_definition.environment().to_vec(),
                        function_definition.arguments().to_vec(),
                        function_definition.body().rename_variables(names),
                        function_definition.result_type().clone(),
                    )
                    .into(),
                    Definition::ValueDefinition(value_definition) => ValueDefinition::new(
                        names
                            .get(value_definition.name())
                            .cloned()
                            .unwrap_or_else(|| value_definition.name().into()),
                        value_definition.body().rename_variables(names),
                        value_definition.type_().clone(),
                    )
                    .into(),
                })
                .collect(),
        )
    }

    pub fn infer_environment(&self) -> Self {
        let global_variables = self
            .declarations
            .iter()
            .map(|declaration| declaration.name().into())
            .chain(
                self.definitions
                    .iter()
                    .map(|definition| definition.name().into()),
            )
            .collect();

        Self::new(
            self.declarations.to_vec(),
            self.definitions
                .iter()
                .map(|definition| {
                    definition.infer_environment(&Default::default(), &global_variables)
                })
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::super::expression::Expression;
    use super::*;
    use crate::ast::*;
    use crate::types;

    #[test]
    fn rename_global_definitions() {
        assert_eq!(
            Module::new(vec![], vec![]).rename_global_definitions(&Default::default()),
            Module::new(vec![], vec![])
        );
        assert_eq!(
            Module::new(
                vec![],
                vec![ValueDefinition::new(
                    "foo",
                    Expression::Number(42.0),
                    types::Value::Number.into()
                )
                .into()]
            )
            .rename_global_definitions(&vec![("foo".into(), "bar".into())].drain(..).collect()),
            Module::new(
                vec![],
                vec![ValueDefinition::new(
                    "bar",
                    Expression::Number(42.0),
                    types::Value::Number.into()
                )
                .into()]
            )
        );
    }

    #[test]
    fn infer_environment() {
        assert_eq!(
            Module::new(
                vec![],
                vec![FunctionDefinition::new(
                    "f",
                    vec![],
                    vec![Argument::new("x", types::Value::Number)],
                    Expression::Number(42.0),
                    types::Value::Number
                )
                .into()]
            )
            .infer_environment(),
            Module::new(
                vec![],
                vec![FunctionDefinition::new(
                    "f",
                    vec![],
                    vec![Argument::new("x", types::Value::Number)],
                    Expression::Number(42.0),
                    types::Value::Number
                )
                .into()]
            )
        );
        assert_eq!(
            Module::new(
                vec![],
                vec![
                    ValueDefinition::new("y", Expression::Number(42.0), types::Value::Number)
                        .into(),
                    FunctionDefinition::new(
                        "f",
                        vec![],
                        vec![Argument::new("x", types::Value::Number)],
                        Variable::new("y"),
                        types::Value::Number
                    )
                    .into()
                ]
            )
            .infer_environment(),
            Module::new(
                vec![],
                vec![
                    ValueDefinition::new("y", Expression::Number(42.0), types::Value::Number)
                        .into(),
                    FunctionDefinition::new(
                        "f",
                        vec![],
                        vec![Argument::new("x", types::Value::Number)],
                        Variable::new("y"),
                        types::Value::Number
                    )
                    .into()
                ]
            )
        );
        assert_eq!(
            Module::new(
                vec![],
                vec![
                    ValueDefinition::new("y", Expression::Number(42.0), types::Value::Number)
                        .into(),
                    FunctionDefinition::new(
                        "f",
                        vec![],
                        vec![Argument::new("x", types::Value::Number)],
                        LetFunctions::new(
                            vec![FunctionDefinition::new(
                                "g",
                                vec![],
                                vec![Argument::new("y", types::Value::Number)],
                                Variable::new("x"),
                                types::Value::Number
                            )],
                            Expression::Number(42.0)
                        ),
                        types::Value::Number
                    )
                    .into()
                ]
            )
            .infer_environment(),
            Module::new(
                vec![],
                vec![
                    ValueDefinition::new("y", Expression::Number(42.0), types::Value::Number)
                        .into(),
                    FunctionDefinition::new(
                        "f",
                        vec![],
                        vec![Argument::new("x", types::Value::Number)],
                        LetFunctions::new(
                            vec![FunctionDefinition::new(
                                "g",
                                vec![Argument::new("x", types::Value::Number)],
                                vec![Argument::new("y", types::Value::Number)],
                                Variable::new("x"),
                                types::Value::Number
                            )],
                            Expression::Number(42.0)
                        ),
                        types::Value::Number
                    )
                    .into()
                ]
            )
        );
    }
}
