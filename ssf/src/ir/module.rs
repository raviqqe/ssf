use super::declaration::Declaration;
use super::definition::Definition;
use crate::analysis::{check_types, AnalysisError};
use crate::types::canonicalize;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub struct Module {
    declarations: Vec<Declaration>,
    definitions: Vec<Definition>,
}

impl Module {
    pub fn new(
        declarations: Vec<Declaration>,
        definitions: Vec<Definition>,
    ) -> Result<Self, AnalysisError> {
        let module = Self {
            declarations,
            definitions: definitions
                .iter()
                .map(|definition| definition.infer_environment(&Default::default()))
                .collect(),
        }
        .canonicalize_types();

        check_types(&module)?;

        Ok(module)
    }

    #[cfg(test)]
    pub fn without_validation(
        declarations: Vec<Declaration>,
        definitions: Vec<Definition>,
    ) -> Self {
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
                    Definition::with_options(
                        names
                            .get(definition.name())
                            .cloned()
                            .unwrap_or_else(|| definition.name().into()),
                        definition.environment().to_vec(),
                        definition.arguments().to_vec(),
                        {
                            let mut names = names.clone();

                            for argument in definition.arguments() {
                                names.remove(argument.name());
                            }

                            definition.body().rename_variables(&names)
                        },
                        definition.result_type().clone(),
                        definition.is_thunk(),
                    )
                })
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
    fn rename_global_values() {
        assert_eq!(
            Module::new(vec![], vec![])
                .unwrap()
                .rename_global_variables(&Default::default()),
            Module::without_validation(vec![], vec![])
        );
        assert_eq!(
            Module::new(
                vec![],
                vec![Definition::new(
                    "foo",
                    vec![Argument::new("x", types::Primitive::Float64)],
                    42.0,
                    types::Primitive::Float64
                )]
            )
            .unwrap()
            .rename_global_variables(&vec![("foo".into(), "bar".into())].drain(..).collect()),
            Module::without_validation(
                vec![],
                vec![Definition::new(
                    "bar",
                    vec![Argument::new("x", types::Primitive::Float64)],
                    42.0,
                    types::Primitive::Float64
                )],
            )
        );
    }

    #[test]
    fn rename_declarations() {
        assert_eq!(
            Module::new(
                vec![Declaration::new(
                    "foo",
                    types::Function::new(
                        vec![types::Primitive::Float64.into()],
                        types::Primitive::Float64
                    )
                )],
                vec![]
            )
            .unwrap()
            .rename_global_variables(&vec![("foo".into(), "bar".into())].drain(..).collect()),
            Module::without_validation(
                vec![Declaration::new(
                    "bar",
                    types::Function::new(
                        vec![types::Primitive::Float64.into()],
                        types::Primitive::Float64
                    )
                )],
                vec![],
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
                    vec![Argument::new("foo", types::Primitive::Float64)],
                    Variable::new("foo"),
                    types::Primitive::Float64
                )]
            )
            .unwrap()
            .rename_global_variables(&vec![("foo".into(), "bar".into())].drain(..).collect()),
            Module::without_validation(
                vec![],
                vec![Definition::new(
                    "bar",
                    vec![Argument::new("foo", types::Primitive::Float64)],
                    Variable::new("foo"),
                    types::Primitive::Float64
                )],
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
                    vec![Argument::new("x", types::Primitive::Float64)],
                    LetRecursive::new(
                        vec![Definition::new(
                            "g",
                            vec![Argument::new("y", types::Primitive::Float64)],
                            Variable::new("x"),
                            types::Primitive::Float64
                        )],
                        42.0
                    ),
                    types::Primitive::Float64
                )]
            )
            .unwrap()
            .rename_global_variables(&Default::default()),
            Module::without_validation(
                vec![],
                vec![Definition::with_environment(
                    "f",
                    vec![],
                    vec![Argument::new("x", types::Primitive::Float64)],
                    LetRecursive::new(
                        vec![Definition::with_environment(
                            "g",
                            vec![Argument::new("x", types::Primitive::Float64)],
                            vec![Argument::new("y", types::Primitive::Float64)],
                            Variable::new("x"),
                            types::Primitive::Float64
                        )],
                        42.0
                    ),
                    types::Primitive::Float64
                )],
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
                    vec![Argument::new("x", types::Primitive::Float64)],
                    42.0,
                    types::Primitive::Float64
                )]
            ),
            Ok(Module::without_validation(
                vec![],
                vec![Definition::with_environment(
                    "f",
                    vec![],
                    vec![Argument::new("x", types::Primitive::Float64)],
                    42.0,
                    types::Primitive::Float64
                )],
            ))
        );
    }

    #[test]
    fn infer_empty_environment_of_global_function_using_global_variable() {
        assert_eq!(
            Module::new(
                vec![],
                vec![
                    Definition::new(
                        "g",
                        vec![Argument::new("x", types::Primitive::Float64)],
                        42.0,
                        types::Primitive::Float64
                    ),
                    Definition::new(
                        "f",
                        vec![Argument::new("x", types::Primitive::Float64)],
                        FunctionApplication::new(Variable::new("g"), vec![42.0.into()]),
                        types::Primitive::Float64
                    )
                ]
            ),
            Ok(Module::without_validation(
                vec![],
                vec![
                    Definition::with_environment(
                        "g",
                        vec![],
                        vec![Argument::new("x", types::Primitive::Float64)],
                        42.0,
                        types::Primitive::Float64
                    ),
                    Definition::with_environment(
                        "f",
                        vec![],
                        vec![Argument::new("x", types::Primitive::Float64)],
                        FunctionApplication::new(Variable::new("g"), vec![42.0.into()]),
                        types::Primitive::Float64
                    )
                ],
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
                    vec![Argument::new("x", types::Primitive::Float64)],
                    LetRecursive::new(
                        vec![Definition::new(
                            "g",
                            vec![Argument::new("y", types::Primitive::Float64)],
                            Variable::new("x"),
                            types::Primitive::Float64
                        )],
                        42.0
                    ),
                    types::Primitive::Float64
                )],
            ),
            Ok(Module::without_validation(
                vec![],
                vec![Definition::with_environment(
                    "f",
                    vec![],
                    vec![Argument::new("x", types::Primitive::Float64)],
                    LetRecursive::new(
                        vec![Definition::with_environment(
                            "g",
                            vec![Argument::new("x", types::Primitive::Float64)],
                            vec![Argument::new("y", types::Primitive::Float64)],
                            Variable::new("x"),
                            types::Primitive::Float64
                        )],
                        42.0
                    ),
                    types::Primitive::Float64
                )],
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
                    vec![Argument::new("x", types::Primitive::Float64)],
                    LetRecursive::new(
                        vec![Definition::new(
                            "g",
                            vec![Argument::new("y", types::Primitive::Float64)],
                            FunctionApplication::new(
                                Variable::new("g"),
                                vec![Variable::new("y").into()]
                            ),
                            types::Primitive::Float64
                        )],
                        42.0
                    ),
                    types::Primitive::Float64
                )]
            ),
            Ok(Module::without_validation(
                vec![],
                vec![Definition::new(
                    "f",
                    vec![Argument::new("x", types::Primitive::Float64)],
                    LetRecursive::new(
                        vec![Definition::with_environment(
                            "g",
                            vec![Argument::new(
                                "g",
                                types::Function::new(
                                    vec![types::Primitive::Float64.into()],
                                    types::Primitive::Float64
                                )
                            )],
                            vec![Argument::new("y", types::Primitive::Float64)],
                            FunctionApplication::new(
                                Variable::new("g"),
                                vec![Variable::new("y").into()]
                            ),
                            types::Primitive::Float64
                        )],
                        42.0
                    ),
                    types::Primitive::Float64
                )],
            ))
        );
    }
}
