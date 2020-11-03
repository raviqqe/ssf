use super::declaration::Declaration;
use super::definition::Definition;
use crate::analysis::{check_types, sort_global_variables, AnalysisError};
use crate::types::canonicalize;
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
        let mut module = Self {
            declarations,
            definitions: definitions
                .iter()
                .map(|definition| definition.infer_environment(&Default::default()))
                .collect(),
            global_variable_initialization_order: vec![],
        }
        .canonicalize_types();

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

    pub fn rename_global_variables(&self, names: &HashMap<String, String>) -> Self {
        Self {
            declarations: self
                .declarations
                .iter()
                .map(|declaration| {
                    Declaration::new(
                        names
                            .get(declaration.name())
                            .cloned()
                            .unwrap_or_else(|| declaration.name().into()),
                        declaration.type_().clone(),
                    )
                })
                .collect(),
            definitions: self
                .definitions
                .iter()
                .map(|definition| {
                    Definition::new(
                        names
                            .get(definition.name())
                            .cloned()
                            .unwrap_or_else(|| definition.name().into()),
                        definition.body().rename_variables(names),
                        definition.type_().clone(),
                    )
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

    fn canonicalize_types(&self) -> Self {
        Self {
            declarations: self
                .declarations
                .iter()
                .map(|declaration| declaration.convert_types(&canonicalize))
                .collect(),
            definitions: self
                .definitions
                .iter()
                .map(|definition| definition.convert_types(&canonicalize))
                .collect(),
            global_variable_initialization_order: self.global_variable_initialization_order.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::*;
    use crate::types;
    use pretty_assertions::assert_eq;

    #[test]
    fn rename_global_types() {
        assert_eq!(
            Module::new(vec![], vec![])
                .unwrap()
                .rename_global_variables(&Default::default()),
            Module::without_validation(vec![], vec![], vec![])
        );
        assert_eq!(
            Module::new(
                vec![],
                vec![Definition::new("foo", 42.0, types::Primitive::Float64).into()]
            )
            .unwrap()
            .rename_global_variables(&vec![("foo".into(), "bar".into())].drain(..).collect()),
            Module::without_validation(
                vec![],
                vec![Definition::new("bar", 42.0, types::Primitive::Float64).into()],
                vec!["bar".into()]
            )
        );
    }

    #[test]
    fn rename_declarations() {
        assert_eq!(
            Module::new(
                vec![Declaration::new("foo", types::Primitive::Float64)],
                vec![]
            )
            .unwrap()
            .rename_global_variables(&vec![("foo".into(), "bar".into())].drain(..).collect()),
            Module::without_validation(
                vec![Declaration::new("bar", types::Primitive::Float64)],
                vec![],
                vec![]
            )
        );
    }

    #[test]
    fn rename_global_functions() {
        assert_eq!(
            Module::new(
                vec![],
                vec![Definition::new(
                    "foo",
                    Lambda::new(
                        vec![Argument::new("foo", types::Primitive::Float64)],
                        Variable::new("foo"),
                        types::Primitive::Float64
                    ),
                    types::Function::new(types::Primitive::Float64, types::Primitive::Float64)
                )]
            )
            .unwrap()
            .rename_global_variables(&vec![("foo".into(), "bar".into())].drain(..).collect()),
            Module::without_validation(
                vec![],
                vec![Definition::new(
                    "bar",
                    Lambda::new(
                        vec![Argument::new("foo", types::Primitive::Float64)],
                        Variable::new("foo"),
                        types::Primitive::Float64
                    ),
                    types::Function::new(types::Primitive::Float64, types::Primitive::Float64)
                )],
                vec!["bar".into()]
            )
        );
    }

    #[test]
    fn do_not_infer_environment_while_renaming_global_definitions() {
        assert_eq!(
            Module::new(
                vec![],
                vec![Definition::new(
                    "f",
                    Lambda::new(
                        vec![Argument::new("x", types::Primitive::Float64)],
                        Let::new(
                            vec![Definition::new(
                                "g",
                                Lambda::new(
                                    vec![Argument::new("y", types::Primitive::Float64)],
                                    Variable::new("x"),
                                    types::Primitive::Float64
                                ),
                                types::Function::new(
                                    types::Primitive::Float64,
                                    types::Primitive::Float64
                                )
                            )],
                            42.0
                        ),
                        types::Primitive::Float64
                    ),
                    types::Function::new(types::Primitive::Float64, types::Primitive::Float64)
                )]
            )
            .unwrap()
            .rename_global_variables(&Default::default()),
            Module::without_validation(
                vec![],
                vec![Definition::new(
                    "f",
                    Lambda::with_environment(
                        vec![],
                        vec![Argument::new("x", types::Primitive::Float64)],
                        Let::new(
                            vec![Definition::new(
                                "g",
                                Lambda::with_environment(
                                    vec![Argument::new("x", types::Primitive::Float64)],
                                    vec![Argument::new("y", types::Primitive::Float64)],
                                    Variable::new("x"),
                                    types::Primitive::Float64
                                ),
                                types::Function::new(
                                    types::Primitive::Float64,
                                    types::Primitive::Float64
                                )
                            )],
                            42.0
                        ),
                        types::Primitive::Float64
                    ),
                    types::Function::new(types::Primitive::Float64, types::Primitive::Float64)
                )],
                vec!["f".into()]
            )
        );
    }

    #[test]
    fn infer_empty_environment_of_global_function() {
        assert_eq!(
            Module::new(
                vec![],
                vec![Definition::new(
                    "f",
                    Lambda::new(
                        vec![Argument::new("x", types::Primitive::Float64)],
                        42.0,
                        types::Primitive::Float64
                    ),
                    types::Function::new(types::Primitive::Float64, types::Primitive::Float64)
                )]
            ),
            Ok(Module::without_validation(
                vec![],
                vec![Definition::new(
                    "f",
                    Lambda::with_environment(
                        vec![],
                        vec![Argument::new("x", types::Primitive::Float64)],
                        42.0,
                        types::Primitive::Float64
                    ),
                    types::Function::new(types::Primitive::Float64, types::Primitive::Float64)
                )],
                vec!["f".into()]
            ))
        );
    }

    #[test]
    fn infer_empty_environment_of_global_function_using_global_variable() {
        assert_eq!(
            Module::new(
                vec![],
                vec![
                    Definition::new("y", 42.0, types::Primitive::Float64).into(),
                    Definition::new(
                        "f",
                        Lambda::new(
                            vec![Argument::new("x", types::Primitive::Float64)],
                            Variable::new("y"),
                            types::Primitive::Float64
                        ),
                        types::Function::new(types::Primitive::Float64, types::Primitive::Float64)
                    )
                ]
            ),
            Ok(Module::without_validation(
                vec![],
                vec![
                    Definition::new("y", 42.0, types::Primitive::Float64).into(),
                    Definition::new(
                        "f",
                        Lambda::with_environment(
                            vec![],
                            vec![Argument::new("x", types::Primitive::Float64)],
                            Variable::new("y"),
                            types::Primitive::Float64
                        ),
                        types::Function::new(types::Primitive::Float64, types::Primitive::Float64)
                    )
                ],
                vec!["f".into(), "y".into()]
            ))
        );
    }

    #[test]
    fn infer_environment_with_captured_argument() {
        assert_eq!(
            Module::new(
                vec![],
                vec![Definition::new(
                    "f",
                    Lambda::new(
                        vec![Argument::new("x", types::Primitive::Float64)],
                        Let::new(
                            vec![Definition::new(
                                "g",
                                Lambda::new(
                                    vec![Argument::new("y", types::Primitive::Float64)],
                                    Variable::new("x"),
                                    types::Primitive::Float64
                                ),
                                types::Function::new(
                                    types::Primitive::Float64,
                                    types::Primitive::Float64
                                )
                            )],
                            42.0
                        ),
                        types::Primitive::Float64
                    ),
                    types::Function::new(types::Primitive::Float64, types::Primitive::Float64)
                )],
            ),
            Ok(Module::without_validation(
                vec![],
                vec![Definition::new(
                    "f",
                    Lambda::with_environment(
                        vec![],
                        vec![Argument::new("x", types::Primitive::Float64)],
                        Let::new(
                            vec![Definition::new(
                                "g",
                                Lambda::with_environment(
                                    vec![Argument::new("x", types::Primitive::Float64)],
                                    vec![Argument::new("y", types::Primitive::Float64)],
                                    Variable::new("x"),
                                    types::Primitive::Float64
                                ),
                                types::Function::new(
                                    types::Primitive::Float64,
                                    types::Primitive::Float64
                                )
                            )],
                            42.0
                        ),
                        types::Primitive::Float64
                    ),
                    types::Function::new(types::Primitive::Float64, types::Primitive::Float64)
                )],
                vec!["f".into()]
            ))
        );
    }

    #[test]
    fn infer_environment_of_recursive_function_in_let_expression() {
        assert_eq!(
            Module::new(
                vec![],
                vec![Definition::new(
                    "f",
                    Lambda::new(
                        vec![Argument::new("x", types::Primitive::Float64)],
                        LetRecursive::new(
                            vec![Definition::new(
                                "g",
                                Lambda::new(
                                    vec![Argument::new("y", types::Primitive::Float64)],
                                    FunctionApplication::new(
                                        Variable::new("g"),
                                        Variable::new("y")
                                    ),
                                    types::Primitive::Float64
                                ),
                                types::Function::new(
                                    types::Primitive::Float64,
                                    types::Primitive::Float64
                                )
                            )],
                            42.0
                        ),
                        types::Primitive::Float64
                    ),
                    types::Function::new(types::Primitive::Float64, types::Primitive::Float64)
                )]
            ),
            Ok(Module::without_validation(
                vec![],
                vec![Definition::new(
                    "f",
                    Lambda::new(
                        vec![Argument::new("x", types::Primitive::Float64)],
                        LetRecursive::new(
                            vec![Definition::new(
                                "g",
                                Lambda::with_environment(
                                    vec![Argument::new(
                                        "g",
                                        types::Function::new(
                                            types::Primitive::Float64,
                                            types::Primitive::Float64
                                        )
                                    )],
                                    vec![Argument::new("y", types::Primitive::Float64)],
                                    FunctionApplication::new(
                                        Variable::new("g"),
                                        Variable::new("y")
                                    ),
                                    types::Primitive::Float64
                                ),
                                types::Function::new(
                                    types::Primitive::Float64,
                                    types::Primitive::Float64
                                )
                            )],
                            42.0
                        ),
                        types::Primitive::Float64
                    ),
                    types::Function::new(types::Primitive::Float64, types::Primitive::Float64)
                )],
                vec!["f".into()]
            ))
        );
    }
}
