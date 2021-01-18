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

pub fn compile(module: &ssf::ir::Module) -> fmm::ir::Module {
    let module_builder = fmm::build::ModuleBuilder::new();

    for declaration in module.foreign_declarations() {
        compile_foreign_declaration(&module_builder, declaration);
    }

    for declaration in module.declarations() {
        compile_declaration(&module_builder, declaration);
    }

    for definition in module.definitions() {
        compile_definition(&module_builder, definition);
    }

    module_builder.as_module()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_compile(module: &ssf::ir::Module) {
        println!("{}", fmm_c::compile(&compile(module)));
    }

    #[test]
    fn compile_empty_module() {
        test_compile(&ssf::ir::Module::new(vec![], vec![], vec![]));
    }

    mod foreign_declarations {
        use super::*;

        #[test]
        fn compile() {
            test_compile(&ssf::ir::Module::new(
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
            test_compile(&ssf::ir::Module::new(
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
            test_compile(&ssf::ir::Module::new(
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
            test_compile(&ssf::ir::Module::new(
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
            test_compile(&ssf::ir::Module::new(
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
            test_compile(&ssf::ir::Module::new(
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
}
