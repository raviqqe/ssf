mod closures;
mod declarations;
mod definitions;
mod entry_functions;
mod error;
mod expressions;
mod foreign_declarations;
mod foreign_definitions;
mod function_applications;
mod types;
mod utilities;
mod variable_builder;

use declarations::compile_declaration;
use definitions::compile_definition;
pub use error::CompileError;
use foreign_declarations::compile_foreign_declaration;
use foreign_definitions::compile_foreign_definition;
use std::collections::HashMap;
use variable_builder::VariableBuilder;

pub fn compile(module: &ssf::ir::Module) -> Result<fmm::ir::Module, CompileError> {
    ssf::analysis::check_types(module)?;

    let module_builder = fmm::build::ModuleBuilder::new();

    for declaration in module.foreign_declarations() {
        compile_foreign_declaration(&module_builder, declaration)?;
    }

    for declaration in module.declarations() {
        compile_declaration(&module_builder, declaration);
    }

    let global_variables = compile_global_variables(module);

    for definition in module.definitions() {
        compile_definition(&module_builder, definition, &global_variables)?;
    }

    let types = module
        .foreign_declarations()
        .iter()
        .map(|declaration| (declaration.name(), declaration.type_()))
        .chain(
            module
                .declarations()
                .iter()
                .map(|declaration| (declaration.name(), declaration.type_())),
        )
        .chain(
            module
                .definitions()
                .iter()
                .map(|definition| (definition.name(), definition.type_())),
        )
        .collect::<HashMap<_, _>>();

    for definition in module.foreign_definitions() {
        compile_foreign_definition(
            &module_builder,
            definition,
            types[definition.name()],
            &global_variables[definition.name()],
        )?;
    }

    Ok(module_builder.as_module())
}

fn compile_global_variables(module: &ssf::ir::Module) -> HashMap<String, VariableBuilder> {
    module
        .foreign_declarations()
        .iter()
        .map(|declaration| {
            (
                declaration.name().into(),
                fmm::build::variable(
                    declaration.name(),
                    fmm::types::Pointer::new(types::compile_unsized_closure(declaration.type_())),
                )
                .into(),
            )
        })
        .chain(module.declarations().iter().map(|declaration| {
            (
                declaration.name().into(),
                fmm::build::variable(
                    declaration.name(),
                    fmm::types::Pointer::new(types::compile_unsized_closure(declaration.type_())),
                )
                .into(),
            )
        }))
        .chain(module.definitions().iter().map(|definition| {
            (
                definition.name().into(),
                VariableBuilder::with_type(
                    fmm::build::variable(
                        definition.name(),
                        fmm::types::Pointer::new(types::compile_sized_closure(definition)),
                    ),
                    fmm::types::Pointer::new(types::compile_unsized_closure(definition.type_())),
                ),
            )
        }))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compile_module(module: &ssf::ir::Module) {
        let module = compile(module).unwrap();

        compile_final_module(&module);
        compile_final_module(
            &fmm::analysis::transform_to_cps(&module, fmm::types::Record::new(vec![])).unwrap(),
        );

        fmm_llvm::compile_to_object(
            &module,
            &fmm_llvm::HeapConfiguration {
                allocate_function_name: "allocate_heap".into(),
                reallocate_function_name: "reallocate_heap".into(),
                free_function_name: "free_heap".into(),
            },
            None,
        )
        .unwrap();
    }

    fn compile_final_module(module: &fmm::ir::Module) {
        let directory = tempfile::tempdir().unwrap();
        let file_path = directory.path().join("foo.c");
        let source = fmm_c::compile(&module, None);

        std::fs::write(&file_path, source).unwrap();
        let output = std::process::Command::new("clang")
            .arg("-Werror") // cspell:disable-line
            .arg("-Wno-incompatible-pointer-types-discards-qualifiers") // cspell:disable-line
            .arg("-o")
            .arg(directory.path().join("foo.o"))
            .arg("-c")
            .arg(&file_path)
            .output()
            .unwrap();

        assert_eq!(String::from_utf8_lossy(&output.stdout), "");
        assert_eq!(String::from_utf8_lossy(&output.stderr), "");
        assert!(output.status.success());
    }

    #[test]
    fn compile_empty_module() {
        compile_module(&ssf::ir::Module::new(vec![], vec![], vec![], vec![]));
    }

    mod foreign_declarations {
        use super::*;

        #[test]
        fn compile() {
            compile_module(&ssf::ir::Module::new(
                vec![ssf::ir::ForeignDeclaration::new(
                    "f",
                    "g",
                    ssf::types::Function::new(
                        ssf::types::Primitive::Float64,
                        ssf::types::Primitive::Float64,
                    ),
                    ssf::ir::CallingConvention::Target,
                )],
                vec![],
                vec![],
                vec![],
            ));
        }

        #[test]
        fn compile_with_multiple_arguments() {
            compile_module(&ssf::ir::Module::new(
                vec![ssf::ir::ForeignDeclaration::new(
                    "f",
                    "g",
                    ssf::types::Function::new(
                        ssf::types::Primitive::Float64,
                        ssf::types::Function::new(
                            ssf::types::Primitive::Float64,
                            ssf::types::Primitive::Float64,
                        ),
                    ),
                    ssf::ir::CallingConvention::Target,
                )],
                vec![],
                vec![],
                vec![],
            ));
        }

        #[test]
        fn compile_with_source_calling_convention() {
            compile_module(&ssf::ir::Module::new(
                vec![ssf::ir::ForeignDeclaration::new(
                    "f",
                    "g",
                    ssf::types::Function::new(
                        ssf::types::Primitive::Float64,
                        ssf::types::Primitive::Float64,
                    ),
                    ssf::ir::CallingConvention::Source,
                )],
                vec![],
                vec![],
                vec![],
            ));
        }
    }

    mod foreign_definitions {
        use super::*;

        #[test]
        fn compile_for_foreign_declaration() {
            compile_module(&ssf::ir::Module::new(
                vec![ssf::ir::ForeignDeclaration::new(
                    "f",
                    "g",
                    ssf::types::Function::new(
                        ssf::types::Primitive::Float64,
                        ssf::types::Primitive::Float64,
                    ),
                    ssf::ir::CallingConvention::Target,
                )],
                vec![ssf::ir::ForeignDefinition::new("f", "h")],
                vec![],
                vec![],
            ));
        }

        #[test]
        fn compile_for_declaration() {
            compile_module(&ssf::ir::Module::new(
                vec![],
                vec![ssf::ir::ForeignDefinition::new("f", "g")],
                vec![ssf::ir::Declaration::new(
                    "f",
                    ssf::types::Function::new(
                        ssf::types::Primitive::Float64,
                        ssf::types::Primitive::Float64,
                    ),
                )],
                vec![],
            ));
        }

        #[test]
        fn compile_for_definition() {
            compile_module(&ssf::ir::Module::new(
                vec![],
                vec![ssf::ir::ForeignDefinition::new("f", "g")],
                vec![],
                vec![ssf::ir::Definition::new(
                    "f",
                    vec![ssf::ir::Argument::new("x", ssf::types::Primitive::Float64)],
                    ssf::ir::Variable::new("x"),
                    ssf::types::Primitive::Float64,
                )],
            ));
        }
    }

    mod declarations {
        use super::*;

        #[test]
        fn compile() {
            compile_module(&ssf::ir::Module::new(
                vec![],
                vec![],
                vec![ssf::ir::Declaration::new(
                    "f",
                    ssf::types::Function::new(
                        ssf::types::Primitive::Float64,
                        ssf::types::Primitive::Float64,
                    ),
                )],
                vec![],
            ));
        }

        #[test]
        fn compile_with_multiple_arguments() {
            compile_module(&ssf::ir::Module::new(
                vec![],
                vec![],
                vec![ssf::ir::Declaration::new(
                    "f",
                    ssf::types::Function::new(
                        ssf::types::Primitive::Float64,
                        ssf::types::Function::new(
                            ssf::types::Primitive::Float64,
                            ssf::types::Primitive::Float64,
                        ),
                    ),
                )],
                vec![],
            ));
        }
    }

    mod definitions {
        use super::*;

        #[test]
        fn compile() {
            compile_module(&ssf::ir::Module::new(
                vec![],
                vec![],
                vec![],
                vec![ssf::ir::Definition::new(
                    "f",
                    vec![ssf::ir::Argument::new("x", ssf::types::Primitive::Float64)],
                    ssf::ir::Variable::new("x"),
                    ssf::types::Primitive::Float64,
                )],
            ));
        }

        #[test]
        fn compile_with_multiple_arguments() {
            compile_module(&ssf::ir::Module::new(
                vec![],
                vec![],
                vec![],
                vec![ssf::ir::Definition::new(
                    "f",
                    vec![
                        ssf::ir::Argument::new("x", ssf::types::Primitive::Float64),
                        ssf::ir::Argument::new("y", ssf::types::Primitive::Float64),
                    ],
                    ssf::ir::ArithmeticOperation::new(
                        ssf::ir::ArithmeticOperator::Add,
                        ssf::ir::Variable::new("x"),
                        ssf::ir::Variable::new("y"),
                    ),
                    ssf::types::Primitive::Float64,
                )],
            ));
        }

        #[test]
        fn compile_thunk() {
            compile_module(&ssf::ir::Module::new(
                vec![],
                vec![],
                vec![],
                vec![
                    ssf::ir::Definition::thunk(
                        "f",
                        vec![ssf::ir::Argument::new("x", ssf::types::Primitive::Float64)],
                        ssf::ir::Variable::new("x"),
                        ssf::types::Primitive::Float64,
                    ),
                    ssf::ir::Definition::new(
                        "g",
                        vec![ssf::ir::Argument::new("x", ssf::types::Primitive::Float64)],
                        ssf::ir::FunctionApplication::new(
                            ssf::ir::Variable::new("f"),
                            ssf::ir::Variable::new("x"),
                        ),
                        ssf::types::Primitive::Float64,
                    ),
                ],
            ));
        }
    }

    mod expressions {
        use super::*;

        #[test]
        fn compile_let() {
            compile_module(&ssf::ir::Module::new(
                vec![],
                vec![],
                vec![],
                vec![ssf::ir::Definition::new(
                    "f",
                    vec![ssf::ir::Argument::new("x", ssf::types::Primitive::Float64)],
                    ssf::ir::Let::new(
                        "y",
                        ssf::types::Primitive::Float64,
                        ssf::ir::Variable::new("x"),
                        ssf::ir::Variable::new("y"),
                    ),
                    ssf::types::Primitive::Float64,
                )],
            ));
        }

        #[test]
        fn compile_let_recursive() {
            compile_module(&ssf::ir::Module::new(
                vec![],
                vec![],
                vec![],
                vec![ssf::ir::Definition::new(
                    "f",
                    vec![ssf::ir::Argument::new("x", ssf::types::Primitive::Float64)],
                    ssf::ir::LetRecursive::new(
                        vec![ssf::ir::Definition::new(
                            "g",
                            vec![ssf::ir::Argument::new("y", ssf::types::Primitive::Float64)],
                            ssf::ir::ArithmeticOperation::new(
                                ssf::ir::ArithmeticOperator::Add,
                                ssf::ir::Variable::new("x"),
                                ssf::ir::Variable::new("y"),
                            ),
                            ssf::types::Primitive::Float64,
                        )],
                        ssf::ir::FunctionApplication::new(
                            ssf::ir::Variable::new("g"),
                            ssf::ir::Primitive::Float64(42.0),
                        ),
                    ),
                    ssf::types::Primitive::Float64,
                )],
            ));
        }

        #[test]
        fn compile_let_recursive_with_curried_function() {
            compile_module(&ssf::ir::Module::new(
                vec![],
                vec![],
                vec![],
                vec![ssf::ir::Definition::new(
                    "f",
                    vec![ssf::ir::Argument::new("x", ssf::types::Primitive::Float64)],
                    ssf::ir::LetRecursive::new(
                        vec![ssf::ir::Definition::new(
                            "g",
                            vec![ssf::ir::Argument::new("y", ssf::types::Primitive::Float64)],
                            ssf::ir::LetRecursive::new(
                                vec![ssf::ir::Definition::new(
                                    "h",
                                    vec![ssf::ir::Argument::new(
                                        "z",
                                        ssf::types::Primitive::Float64,
                                    )],
                                    ssf::ir::ArithmeticOperation::new(
                                        ssf::ir::ArithmeticOperator::Add,
                                        ssf::ir::ArithmeticOperation::new(
                                            ssf::ir::ArithmeticOperator::Add,
                                            ssf::ir::Variable::new("x"),
                                            ssf::ir::Variable::new("y"),
                                        ),
                                        ssf::ir::Variable::new("z"),
                                    ),
                                    ssf::types::Primitive::Float64,
                                )],
                                ssf::ir::Variable::new("h"),
                            ),
                            ssf::types::Function::new(
                                ssf::types::Primitive::Float64,
                                ssf::types::Primitive::Float64,
                            ),
                        )],
                        ssf::ir::FunctionApplication::new(
                            ssf::ir::FunctionApplication::new(
                                ssf::ir::Variable::new("g"),
                                ssf::ir::Primitive::Float64(42.0),
                            ),
                            ssf::ir::Primitive::Float64(42.0),
                        ),
                    ),
                    ssf::types::Primitive::Float64,
                )],
            ));
        }

        mod algebraic_cases {
            use super::*;

            #[test]
            fn compile_with_singleton_enum() {
                let algebraic_type =
                    ssf::types::Algebraic::new(vec![ssf::types::Constructor::unboxed(vec![])]);

                compile_module(&ssf::ir::Module::new(
                    vec![],
                    vec![],
                    vec![],
                    vec![ssf::ir::Definition::new(
                        "f",
                        vec![ssf::ir::Argument::new("x", algebraic_type.clone())],
                        ssf::ir::AlgebraicCase::new(
                            ssf::ir::Variable::new("x"),
                            vec![ssf::ir::AlgebraicAlternative::new(
                                ssf::ir::Constructor::new(algebraic_type, 0),
                                vec![],
                                ssf::ir::Primitive::Float64(42.0),
                            )],
                            None,
                        ),
                        ssf::types::Primitive::Float64,
                    )],
                ));
            }

            #[test]
            fn compile_with_1_element_singleton() {
                let algebraic_type = ssf::types::Algebraic::new(vec![
                    ssf::types::Constructor::unboxed(vec![ssf::types::Primitive::Float64.into()]),
                ]);

                compile_module(&ssf::ir::Module::new(
                    vec![],
                    vec![],
                    vec![],
                    vec![ssf::ir::Definition::new(
                        "f",
                        vec![ssf::ir::Argument::new("x", algebraic_type.clone())],
                        ssf::ir::AlgebraicCase::new(
                            ssf::ir::Variable::new("x"),
                            vec![ssf::ir::AlgebraicAlternative::new(
                                ssf::ir::Constructor::new(algebraic_type, 0),
                                vec!["y".into()],
                                ssf::ir::Variable::new("y"),
                            )],
                            None,
                        ),
                        ssf::types::Primitive::Float64,
                    )],
                ));
            }

            #[test]
            fn compile_with_boxed_1_element_singleton() {
                let algebraic_type = ssf::types::Algebraic::new(vec![
                    ssf::types::Constructor::boxed(vec![ssf::types::Primitive::Float64.into()]),
                ]);

                compile_module(&ssf::ir::Module::new(
                    vec![],
                    vec![],
                    vec![],
                    vec![ssf::ir::Definition::new(
                        "f",
                        vec![ssf::ir::Argument::new("x", algebraic_type.clone())],
                        ssf::ir::AlgebraicCase::new(
                            ssf::ir::Variable::new("x"),
                            vec![ssf::ir::AlgebraicAlternative::new(
                                ssf::ir::Constructor::new(algebraic_type, 0),
                                vec!["y".into()],
                                ssf::ir::Variable::new("y"),
                            )],
                            None,
                        ),
                        ssf::types::Primitive::Float64,
                    )],
                ));
            }

            #[test]
            fn compile_with_2_members_and_1_element() {
                let algebraic_type = ssf::types::Algebraic::new(vec![
                    ssf::types::Constructor::unboxed(vec![ssf::types::Primitive::Float64.into()]),
                    ssf::types::Constructor::unboxed(vec![]),
                ]);

                compile_module(&ssf::ir::Module::new(
                    vec![],
                    vec![],
                    vec![],
                    vec![ssf::ir::Definition::new(
                        "f",
                        vec![ssf::ir::Argument::new("x", algebraic_type.clone())],
                        ssf::ir::AlgebraicCase::new(
                            ssf::ir::Variable::new("x"),
                            vec![ssf::ir::AlgebraicAlternative::new(
                                ssf::ir::Constructor::new(algebraic_type, 0),
                                vec!["y".into()],
                                ssf::ir::Variable::new("y"),
                            )],
                            Some(ssf::ir::Primitive::Float64(42.0).into()),
                        ),
                        ssf::types::Primitive::Float64,
                    )],
                ));
            }

            #[test]
            fn compile_with_custom_tags() {
                let algebraic_type = ssf::types::Algebraic::with_tags(
                    vec![
                        (
                            42,
                            ssf::types::Constructor::unboxed(vec![
                                ssf::types::Primitive::Float64.into()
                            ]),
                        ),
                        (2045, ssf::types::Constructor::unboxed(vec![])),
                    ]
                    .into_iter()
                    .collect(),
                );

                compile_module(&ssf::ir::Module::new(
                    vec![],
                    vec![],
                    vec![],
                    vec![ssf::ir::Definition::new(
                        "f",
                        vec![ssf::ir::Argument::new("x", algebraic_type.clone())],
                        ssf::ir::AlgebraicCase::new(
                            ssf::ir::Variable::new("x"),
                            vec![
                                ssf::ir::AlgebraicAlternative::new(
                                    ssf::ir::Constructor::new(algebraic_type.clone(), 42),
                                    vec!["y".into()],
                                    ssf::ir::Variable::new("y"),
                                ),
                                ssf::ir::AlgebraicAlternative::new(
                                    ssf::ir::Constructor::new(algebraic_type, 2045),
                                    vec![],
                                    ssf::ir::Primitive::Float64(42.0),
                                ),
                            ],
                            None,
                        ),
                        ssf::types::Primitive::Float64,
                    )],
                ));
            }
        }

        mod primitive_cases {
            use super::*;

            #[test]
            fn compile() {
                compile_module(&ssf::ir::Module::new(
                    vec![],
                    vec![],
                    vec![],
                    vec![ssf::ir::Definition::new(
                        "f",
                        vec![ssf::ir::Argument::new("x", ssf::types::Primitive::Float64)],
                        ssf::ir::PrimitiveCase::new(
                            ssf::ir::Variable::new("x"),
                            vec![
                                ssf::ir::PrimitiveAlternative::new(
                                    ssf::ir::Primitive::Float64(0.0),
                                    ssf::ir::Primitive::Float64(1.0),
                                ),
                                ssf::ir::PrimitiveAlternative::new(
                                    ssf::ir::Primitive::Float64(2.0),
                                    ssf::ir::Primitive::Float64(3.0),
                                ),
                            ],
                            None,
                        ),
                        ssf::types::Primitive::Float64,
                    )],
                ));
            }

            #[test]
            fn compile_with_default_alternative() {
                compile_module(&ssf::ir::Module::new(
                    vec![],
                    vec![],
                    vec![],
                    vec![ssf::ir::Definition::new(
                        "f",
                        vec![ssf::ir::Argument::new("x", ssf::types::Primitive::Float64)],
                        ssf::ir::PrimitiveCase::new(
                            ssf::ir::Variable::new("x"),
                            vec![
                                ssf::ir::PrimitiveAlternative::new(
                                    ssf::ir::Primitive::Float64(0.0),
                                    ssf::ir::Primitive::Float64(1.0),
                                ),
                                ssf::ir::PrimitiveAlternative::new(
                                    ssf::ir::Primitive::Float64(2.0),
                                    ssf::ir::Primitive::Float64(3.0),
                                ),
                            ],
                            Some(ssf::ir::Primitive::Float64(4.0).into()),
                        ),
                        ssf::types::Primitive::Float64,
                    )],
                ));
            }
        }

        mod constructor_applications {
            use super::*;

            #[test]
            fn compile_singleton_enum() {
                let algebraic_type =
                    ssf::types::Algebraic::new(vec![ssf::types::Constructor::unboxed(vec![])]);

                compile_module(&ssf::ir::Module::new(
                    vec![],
                    vec![],
                    vec![],
                    vec![ssf::ir::Definition::new(
                        "f",
                        vec![ssf::ir::Argument::new("x", ssf::types::Primitive::Float64)],
                        ssf::ir::ConstructorApplication::new(
                            ssf::ir::Constructor::new(algebraic_type.clone(), 0),
                            vec![],
                        ),
                        algebraic_type,
                    )],
                ));
            }

            #[test]
            fn compile_singleton_with_1_element() {
                let algebraic_type = ssf::types::Algebraic::new(vec![
                    ssf::types::Constructor::unboxed(vec![ssf::types::Primitive::Float64.into()]),
                ]);

                compile_module(&ssf::ir::Module::new(
                    vec![],
                    vec![],
                    vec![],
                    vec![ssf::ir::Definition::new(
                        "f",
                        vec![ssf::ir::Argument::new("x", ssf::types::Primitive::Float64)],
                        ssf::ir::ConstructorApplication::new(
                            ssf::ir::Constructor::new(algebraic_type.clone(), 0),
                            vec![ssf::ir::Primitive::Float64(42.0).into()],
                        ),
                        algebraic_type,
                    )],
                ));
            }

            #[test]
            fn compile_singleton_with_2_elements() {
                let algebraic_type =
                    ssf::types::Algebraic::new(vec![ssf::types::Constructor::unboxed(vec![
                        ssf::types::Primitive::Float64.into(),
                        ssf::types::Primitive::Integer64.into(),
                    ])]);

                compile_module(&ssf::ir::Module::new(
                    vec![],
                    vec![],
                    vec![],
                    vec![ssf::ir::Definition::new(
                        "f",
                        vec![ssf::ir::Argument::new("x", ssf::types::Primitive::Float64)],
                        ssf::ir::ConstructorApplication::new(
                            ssf::ir::Constructor::new(algebraic_type.clone(), 0),
                            vec![
                                ssf::ir::Primitive::Float64(42.0).into(),
                                ssf::ir::Primitive::Integer64(42).into(),
                            ],
                        ),
                        algebraic_type,
                    )],
                ));
            }

            #[test]
            fn compile_boxed_singleton() {
                let algebraic_type = ssf::types::Algebraic::new(vec![
                    ssf::types::Constructor::boxed(vec![ssf::types::Primitive::Float64.into()]),
                ]);

                compile_module(&ssf::ir::Module::new(
                    vec![],
                    vec![],
                    vec![],
                    vec![ssf::ir::Definition::new(
                        "f",
                        vec![ssf::ir::Argument::new("x", ssf::types::Primitive::Float64)],
                        ssf::ir::ConstructorApplication::new(
                            ssf::ir::Constructor::new(algebraic_type.clone(), 0),
                            vec![ssf::ir::Primitive::Float64(42.0).into()],
                        ),
                        algebraic_type,
                    )],
                ));
            }

            #[test]
            fn compile_multiple_members() {
                let algebraic_type = ssf::types::Algebraic::new(vec![
                    ssf::types::Constructor::unboxed(vec![ssf::types::Primitive::Float64.into()]),
                    ssf::types::Constructor::unboxed(vec![ssf::types::Primitive::Integer64.into()]),
                ]);

                compile_module(&ssf::ir::Module::new(
                    vec![],
                    vec![],
                    vec![],
                    vec![ssf::ir::Definition::new(
                        "f",
                        vec![ssf::ir::Argument::new("x", ssf::types::Primitive::Float64)],
                        ssf::ir::ConstructorApplication::new(
                            ssf::ir::Constructor::new(algebraic_type.clone(), 0),
                            vec![ssf::ir::Primitive::Float64(42.0).into()],
                        ),
                        algebraic_type,
                    )],
                ));
            }
        }

        mod function_applications {
            use super::*;

            #[test]
            fn compile_1_argument() {
                compile_module(&ssf::ir::Module::new(
                    vec![],
                    vec![],
                    vec![],
                    vec![
                        ssf::ir::Definition::new(
                            "f",
                            vec![ssf::ir::Argument::new("x", ssf::types::Primitive::Float64)],
                            ssf::ir::Variable::new("x"),
                            ssf::types::Primitive::Float64,
                        ),
                        ssf::ir::Definition::new(
                            "g",
                            vec![ssf::ir::Argument::new("x", ssf::types::Primitive::Float64)],
                            ssf::ir::FunctionApplication::new(
                                ssf::ir::Variable::new("f"),
                                ssf::ir::Primitive::Float64(42.0),
                            ),
                            ssf::types::Primitive::Float64,
                        ),
                    ],
                ));
            }

            #[test]
            fn compile_2_arguments() {
                compile_module(&ssf::ir::Module::new(
                    vec![],
                    vec![],
                    vec![],
                    vec![
                        ssf::ir::Definition::new(
                            "f",
                            vec![
                                ssf::ir::Argument::new("x", ssf::types::Primitive::Float64),
                                ssf::ir::Argument::new("y", ssf::types::Primitive::Integer32),
                            ],
                            ssf::ir::Variable::new("x"),
                            ssf::types::Primitive::Float64,
                        ),
                        ssf::ir::Definition::new(
                            "g",
                            vec![ssf::ir::Argument::new("x", ssf::types::Primitive::Float64)],
                            ssf::ir::FunctionApplication::new(
                                ssf::ir::FunctionApplication::new(
                                    ssf::ir::Variable::new("f"),
                                    ssf::ir::Primitive::Float64(42.0),
                                ),
                                ssf::ir::Primitive::Integer32(42),
                            ),
                            ssf::types::Primitive::Float64,
                        ),
                    ],
                ));
            }

            #[test]
            fn compile_3_arguments() {
                compile_module(&ssf::ir::Module::new(
                    vec![],
                    vec![],
                    vec![],
                    vec![
                        ssf::ir::Definition::new(
                            "f",
                            vec![
                                ssf::ir::Argument::new("x", ssf::types::Primitive::Float64),
                                ssf::ir::Argument::new("y", ssf::types::Primitive::Integer32),
                                ssf::ir::Argument::new("z", ssf::types::Primitive::Integer64),
                            ],
                            ssf::ir::Variable::new("x"),
                            ssf::types::Primitive::Float64,
                        ),
                        ssf::ir::Definition::new(
                            "g",
                            vec![ssf::ir::Argument::new("x", ssf::types::Primitive::Float64)],
                            ssf::ir::FunctionApplication::new(
                                ssf::ir::FunctionApplication::new(
                                    ssf::ir::FunctionApplication::new(
                                        ssf::ir::Variable::new("f"),
                                        ssf::ir::Primitive::Float64(111.0),
                                    ),
                                    ssf::ir::Primitive::Integer32(222),
                                ),
                                ssf::ir::Primitive::Integer64(333),
                            ),
                            ssf::types::Primitive::Float64,
                        ),
                    ],
                ));
            }

            #[test]
            fn compile_1_argument_with_arity_of_2() {
                compile_module(&ssf::ir::Module::new(
                    vec![],
                    vec![],
                    vec![],
                    vec![
                        ssf::ir::Definition::new(
                            "f",
                            vec![
                                ssf::ir::Argument::new("x", ssf::types::Primitive::Float64),
                                ssf::ir::Argument::new("y", ssf::types::Primitive::Integer32),
                            ],
                            ssf::ir::Variable::new("x"),
                            ssf::types::Primitive::Float64,
                        ),
                        ssf::ir::Definition::new(
                            "g",
                            vec![ssf::ir::Argument::new("x", ssf::types::Primitive::Float64)],
                            ssf::ir::FunctionApplication::new(
                                ssf::ir::Variable::new("f"),
                                ssf::ir::Primitive::Float64(42.0),
                            ),
                            ssf::types::Function::new(
                                ssf::types::Primitive::Integer32,
                                ssf::types::Primitive::Float64,
                            ),
                        ),
                    ],
                ));
            }

            #[test]
            fn compile_1_argument_with_arity_of_3() {
                compile_module(&ssf::ir::Module::new(
                    vec![],
                    vec![],
                    vec![],
                    vec![
                        ssf::ir::Definition::new(
                            "f",
                            vec![
                                ssf::ir::Argument::new("x", ssf::types::Primitive::Float64),
                                ssf::ir::Argument::new("y", ssf::types::Primitive::Integer32),
                                ssf::ir::Argument::new("z", ssf::types::Primitive::Integer64),
                            ],
                            ssf::ir::Variable::new("x"),
                            ssf::types::Primitive::Float64,
                        ),
                        ssf::ir::Definition::new(
                            "g",
                            vec![ssf::ir::Argument::new("x", ssf::types::Primitive::Float64)],
                            ssf::ir::FunctionApplication::new(
                                ssf::ir::Variable::new("f"),
                                ssf::ir::Primitive::Float64(42.0),
                            ),
                            ssf::types::Function::new(
                                ssf::types::Primitive::Integer32,
                                ssf::types::Function::new(
                                    ssf::types::Primitive::Integer64,
                                    ssf::types::Primitive::Float64,
                                ),
                            ),
                        ),
                    ],
                ));
            }

            #[test]
            fn compile_2_arguments_with_arity_of_3() {
                compile_module(&ssf::ir::Module::new(
                    vec![],
                    vec![],
                    vec![],
                    vec![
                        ssf::ir::Definition::new(
                            "f",
                            vec![
                                ssf::ir::Argument::new("x", ssf::types::Primitive::Float64),
                                ssf::ir::Argument::new("y", ssf::types::Primitive::Integer32),
                                ssf::ir::Argument::new("z", ssf::types::Primitive::Integer64),
                            ],
                            ssf::ir::Variable::new("x"),
                            ssf::types::Primitive::Float64,
                        ),
                        ssf::ir::Definition::new(
                            "g",
                            vec![ssf::ir::Argument::new("x", ssf::types::Primitive::Float64)],
                            ssf::ir::FunctionApplication::new(
                                ssf::ir::FunctionApplication::new(
                                    ssf::ir::Variable::new("f"),
                                    ssf::ir::Primitive::Float64(111.0),
                                ),
                                ssf::ir::Primitive::Integer32(222),
                            ),
                            ssf::types::Function::new(
                                ssf::types::Primitive::Integer64,
                                ssf::types::Primitive::Float64,
                            ),
                        ),
                    ],
                ));
            }

            #[test]
            fn compile_with_curried_function() {
                compile_module(&ssf::ir::Module::new(
                    vec![],
                    vec![],
                    vec![],
                    vec![
                        ssf::ir::Definition::new(
                            "f",
                            vec![ssf::ir::Argument::new("x", ssf::types::Primitive::Float64)],
                            ssf::ir::LetRecursive::new(
                                vec![ssf::ir::Definition::new(
                                    "g",
                                    vec![ssf::ir::Argument::new(
                                        "y",
                                        ssf::types::Primitive::Float64,
                                    )],
                                    ssf::ir::ArithmeticOperation::new(
                                        ssf::ir::ArithmeticOperator::Add,
                                        ssf::ir::Variable::new("x"),
                                        ssf::ir::Variable::new("y"),
                                    ),
                                    ssf::types::Primitive::Float64,
                                )],
                                ssf::ir::Variable::new("g"),
                            ),
                            ssf::types::Function::new(
                                ssf::types::Primitive::Float64,
                                ssf::types::Primitive::Float64,
                            ),
                        ),
                        ssf::ir::Definition::new(
                            "g",
                            vec![ssf::ir::Argument::new("x", ssf::types::Primitive::Float64)],
                            ssf::ir::FunctionApplication::new(
                                ssf::ir::FunctionApplication::new(
                                    ssf::ir::Variable::new("f"),
                                    ssf::ir::Primitive::Float64(111.0),
                                ),
                                ssf::ir::Primitive::Float64(222.0),
                            ),
                            ssf::types::Primitive::Float64,
                        ),
                    ],
                ));
            }
        }
    }
}
