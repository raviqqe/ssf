use super::declaration::Declaration;
use super::definition::Definition;
use super::foreign_declaration::ForeignDeclaration;
use crate::analysis::{check_types, AnalysisError};
use crate::types::canonicalize;

#[derive(Clone, Debug, PartialEq)]
pub struct Module {
    foreign_declarations: Vec<ForeignDeclaration>,
    declarations: Vec<Declaration>,
    definitions: Vec<Definition>,
}

impl Module {
    pub fn new(
        foreign_declarations: Vec<ForeignDeclaration>,
        declarations: Vec<Declaration>,
        definitions: Vec<Definition>,
    ) -> Result<Self, AnalysisError> {
        let module = Self {
            foreign_declarations,
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
        foreign_declarations: Vec<ForeignDeclaration>,
        declarations: Vec<Declaration>,
        definitions: Vec<Definition>,
    ) -> Self {
        Self {
            foreign_declarations,
            declarations,
            definitions,
        }
    }

    pub fn foreign_declarations(&self) -> &[ForeignDeclaration] {
        &self.foreign_declarations
    }

    pub fn declarations(&self) -> &[Declaration] {
        &self.declarations
    }

    pub fn definitions(&self) -> &[Definition] {
        &self.definitions
    }

    fn canonicalize_types(&self) -> Self {
        Self {
            foreign_declarations: self
                .foreign_declarations
                .iter()
                .map(|declaration| declaration.convert_types(&canonicalize))
                .collect(),
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
    fn infer_empty_environment_of_global_function() {
        assert_eq!(
            Module::new(
                vec![],
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
                        FunctionApplication::new(Variable::new("g"), 42.0),
                        types::Primitive::Float64
                    )
                ]
            ),
            Ok(Module::without_validation(
                vec![],
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
                        FunctionApplication::new(Variable::new("g"), 42.0),
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
                vec![],
                vec![Definition::new(
                    "f",
                    vec![Argument::new("x", types::Primitive::Float64)],
                    LetRecursive::new(
                        vec![Definition::new(
                            "g",
                            vec![Argument::new("y", types::Primitive::Float64)],
                            FunctionApplication::new(Variable::new("g"), Variable::new("y")),
                            types::Primitive::Float64
                        )],
                        42.0
                    ),
                    types::Primitive::Float64
                )]
            ),
            Ok(Module::without_validation(
                vec![],
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
                                    types::Primitive::Float64,
                                    types::Primitive::Float64
                                )
                            )],
                            vec![Argument::new("y", types::Primitive::Float64)],
                            FunctionApplication::new(Variable::new("g"), Variable::new("y")),
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
