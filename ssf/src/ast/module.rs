use super::declaration::Declaration;
use super::definition::Definition;
use super::function_definition::FunctionDefinition;
use super::value_definition::ValueDefinition;
use crate::analysis::{check_types, sort_global_variables, AnalysisError};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub struct Module {
    declarations: Vec<Declaration>,
    definitions: Vec<Definition>,
    global_variable_initialization_order: Vec<String>,
}

impl Module {
    pub fn new(
        declarations: Vec<Declaration>,
        definitions: Vec<Definition>,
    ) -> Result<Self, AnalysisError> {
        let global_variables = declarations
            .iter()
            .map(|declaration| declaration.name().into())
            .chain(
                definitions
                    .iter()
                    .map(|definition| definition.name().into()),
            )
            .collect();

        let mut module = Self {
            declarations,
            definitions: definitions
                .iter()
                .map(|definition| {
                    definition.infer_environment(&Default::default(), &global_variables)
                })
                .collect(),
            global_variable_initialization_order: vec![],
        };

        check_types(&module)?;
        module.global_variable_initialization_order = sort_global_variables(&module)?
            .into_iter()
            .map(|string| string.into())
            .collect();

        Ok(module)
    }

    #[cfg(test)]
    pub fn without_validation(
        declarations: Vec<Declaration>,
        definitions: Vec<Definition>,
        global_variable_initialization_order: Vec<String>,
    ) -> Self {
        Self {
            declarations,
            definitions,
            global_variable_initialization_order,
        }
    }

    pub fn declarations(&self) -> &[Declaration] {
        &self.declarations
    }

    pub fn definitions(&self) -> &[Definition] {
        &self.definitions
    }

    pub fn global_variable_initialization_order(&self) -> &[String] {
        &self.global_variable_initialization_order
    }

    pub fn rename_global_definitions(&self, names: &HashMap<String, String>) -> Self {
        Self {
            declarations: self.declarations.to_vec(),
            definitions: self
                .definitions
                .iter()
                .map(|definition| match definition {
                    Definition::FunctionDefinition(function_definition) => {
                        FunctionDefinition::with_environment(
                            names
                                .get(function_definition.name())
                                .cloned()
                                .unwrap_or_else(|| function_definition.name().into()),
                            function_definition.environment().to_vec(),
                            function_definition.arguments().to_vec(),
                            function_definition.body().rename_variables(names),
                            function_definition.result_type().clone(),
                        )
                        .into()
                    }
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
            global_variable_initialization_order: self
                .global_variable_initialization_order
                .iter()
                .map(|name| names.get(name).unwrap_or_else(|| name))
                .cloned()
                .collect(),
        }
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
            Module::new(vec![], vec![])
                .unwrap()
                .rename_global_definitions(&Default::default()),
            Module::without_validation(vec![], vec![], vec![])
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
            .unwrap()
            .rename_global_definitions(&vec![("foo".into(), "bar".into())].drain(..).collect()),
            Module::without_validation(
                vec![],
                vec![ValueDefinition::new(
                    "bar",
                    Expression::Number(42.0),
                    types::Value::Number.into()
                )
                .into()],
                vec!["bar".into()]
            )
        );
    }

    #[test]
    fn do_not_infer_environment_while_renaming_global_definitions() {
        assert_eq!(
            Module::new(
                vec![],
                vec![
                    ValueDefinition::new("y", Expression::Number(42.0), types::Value::Number)
                        .into(),
                    FunctionDefinition::new(
                        "f",
                        vec![Argument::new("x", types::Value::Number)],
                        LetFunctions::new(
                            vec![FunctionDefinition::new(
                                "g",
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
            .unwrap()
            .rename_global_definitions(&Default::default()),
            Module::without_validation(
                vec![],
                vec![
                    ValueDefinition::new("y", Expression::Number(42.0), types::Value::Number)
                        .into(),
                    FunctionDefinition::with_environment(
                        "f",
                        vec![],
                        vec![Argument::new("x", types::Value::Number)],
                        LetFunctions::new(
                            vec![FunctionDefinition::with_environment(
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
                ],
                vec!["y".into()]
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
                    vec![Argument::new("x", types::Value::Number)],
                    Expression::Number(42.0),
                    types::Value::Number
                )
                .into()]
            ),
            Ok(Module::without_validation(
                vec![],
                vec![FunctionDefinition::with_environment(
                    "f",
                    vec![],
                    vec![Argument::new("x", types::Value::Number)],
                    Expression::Number(42.0),
                    types::Value::Number
                )
                .into()],
                vec![]
            ))
        );
        assert_eq!(
            Module::new(
                vec![],
                vec![
                    ValueDefinition::new("y", Expression::Number(42.0), types::Value::Number)
                        .into(),
                    FunctionDefinition::new(
                        "f",
                        vec![Argument::new("x", types::Value::Number)],
                        Variable::new("y"),
                        types::Value::Number
                    )
                    .into()
                ]
            ),
            Ok(Module::without_validation(
                vec![],
                vec![
                    ValueDefinition::new("y", Expression::Number(42.0), types::Value::Number)
                        .into(),
                    FunctionDefinition::with_environment(
                        "f",
                        vec![],
                        vec![Argument::new("x", types::Value::Number)],
                        Variable::new("y"),
                        types::Value::Number
                    )
                    .into()
                ],
                vec!["y".into()]
            ))
        );
        assert_eq!(
            Module::new(
                vec![],
                vec![
                    ValueDefinition::new("y", Expression::Number(42.0), types::Value::Number)
                        .into(),
                    FunctionDefinition::new(
                        "f",
                        vec![Argument::new("x", types::Value::Number)],
                        LetFunctions::new(
                            vec![FunctionDefinition::new(
                                "g",
                                vec![Argument::new("y", types::Value::Number)],
                                Variable::new("x"),
                                types::Value::Number
                            )],
                            Expression::Number(42.0)
                        ),
                        types::Value::Number
                    )
                    .into()
                ],
            ),
            Ok(Module::without_validation(
                vec![],
                vec![
                    ValueDefinition::new("y", Expression::Number(42.0), types::Value::Number)
                        .into(),
                    FunctionDefinition::with_environment(
                        "f",
                        vec![],
                        vec![Argument::new("x", types::Value::Number)],
                        LetFunctions::new(
                            vec![FunctionDefinition::with_environment(
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
                ],
                vec!["y".into()]
            ))
        );
    }
}
