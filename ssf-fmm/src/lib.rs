mod closures;
mod declarations;
mod definitions;
mod entry_functions;
mod expressions;
mod foreign_declarations;
mod function_applications;
mod types;
mod utilities;

use declarations::compile_declaration;
use definitions::compile_definition;
use foreign_declarations::compile_foreign_declaration;
use std::collections::HashMap;

pub fn compile(module: &ssf::ir::Module) -> fmm::ir::Module {
    let module_builder = fmm::build::ModuleBuilder::new();

    for declaration in module.foreign_declarations() {
        compile_foreign_declaration(&module_builder, declaration);
    }

    for declaration in module.declarations() {
        compile_declaration(&module_builder, declaration);
    }

    let global_variables = compile_global_variables(module);

    for definition in module.definitions() {
        compile_definition(&module_builder, definition, &global_variables);
    }

    module_builder.as_module()
}

fn compile_global_variables(
    module: &ssf::ir::Module,
) -> HashMap<String, fmm::build::TypedExpression> {
    module
        .foreign_declarations()
        .iter()
        .map(|declaration| {
            (
                declaration.name().into(),
                utilities::variable(
                    declaration.name(),
                    fmm::types::Pointer::new(types::compile_unsized_closure(declaration.type_())),
                ),
            )
        })
        .chain(module.declarations().iter().map(|declaration| {
            (
                declaration.name().into(),
                utilities::variable(
                    declaration.name(),
                    fmm::types::Pointer::new(types::compile_unsized_closure(declaration.type_())),
                ),
            )
        }))
        .chain(module.definitions().iter().map(|definition| {
            (
                definition.name().into(),
                utilities::variable(
                    definition.name(),
                    fmm::types::Pointer::new(types::compile_sized_closure(definition)),
                ),
            )
        }))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compile_module(module: &ssf::ir::Module) {
        let directory = tempfile::tempdir().unwrap();
        let file_path = directory.path().join("foo.c");
        let source = fmm_c::compile(&compile(module));

        println!("{}", source);

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
        compile_module(&ssf::ir::Module::new(vec![], vec![], vec![]));
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
                )],
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
                )],
                vec![],
                vec![],
            ));
        }
    }

    mod declarations {
        use super::*;

        #[test]
        fn compile() {
            compile_module(&ssf::ir::Module::new(
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
                vec![ssf::ir::Definition::new(
                    "f",
                    vec![
                        ssf::ir::Argument::new("x", ssf::types::Primitive::Float64),
                        ssf::ir::Argument::new("y", ssf::types::Primitive::Float64),
                    ],
                    ssf::ir::PrimitiveOperation::new(
                        ssf::ir::PrimitiveOperator::Add,
                        ssf::ir::Variable::new("x"),
                        ssf::ir::Variable::new("y"),
                    ),
                    ssf::types::Primitive::Float64,
                )],
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
                vec![ssf::ir::Definition::new(
                    "f",
                    vec![ssf::ir::Argument::new("x", ssf::types::Primitive::Float64)],
                    ssf::ir::LetRecursive::new(
                        vec![ssf::ir::Definition::new(
                            "g",
                            vec![ssf::ir::Argument::new("y", ssf::types::Primitive::Float64)],
                            ssf::ir::PrimitiveOperation::new(
                                ssf::ir::PrimitiveOperator::Add,
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

        mod function_applications {
            use super::*;

            #[test]
            fn compile_one_argument() {
                compile_module(&ssf::ir::Module::new(
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
            fn compile_two_arguments() {
                compile_module(&ssf::ir::Module::new(
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
            fn compile_three_arguments() {
                compile_module(&ssf::ir::Module::new(
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
            fn compile_one_argument_with_arity_of_2() {
                compile_module(&ssf::ir::Module::new(
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
            fn compile_one_arguments_with_arity_of_3() {
                compile_module(&ssf::ir::Module::new(
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
            fn compile_two_arguments_with_arity_of_3() {
                compile_module(&ssf::ir::Module::new(
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
        }
    }
}
