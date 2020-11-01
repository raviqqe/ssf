use super::error::AnalysisError;
use crate::ir;
use petgraph::algo::toposort;
use petgraph::graph::Graph;
use std::collections::{HashMap, HashSet};

pub fn sort_global_variables(module: &ir::Module) -> Result<Vec<String>, AnalysisError> {
    let mut graph = Graph::<String, ()>::new();
    let mut name_indices = HashMap::<String, _>::new();

    for definition in module.definitions() {
        for name in vec![
            definition.name().into(),
            get_indirect_name(definition.name()),
        ] {
            name_indices.insert(name.clone(), graph.add_node(name));
        }
    }

    for definition in module.definitions() {
        for name in definition.find_free_variables(true) {
            graph.add_edge(
                name_indices[name.as_str()],
                name_indices[definition.name()],
                (),
            );

            graph.add_edge(
                name_indices[&get_indirect_name(&name)],
                name_indices[definition.name()],
                (),
            );
        }

        for name in definition.find_free_variables(false) {
            graph.add_edge(
                name_indices[name.as_str()],
                name_indices[&get_indirect_name(definition.name())],
                (),
            );
        }
    }

    let global_names = module
        .definitions()
        .iter()
        .map(|definition| definition.name())
        .collect::<HashSet<&str>>();

    Ok(toposort(&graph, None)?
        .into_iter()
        .map(|index| graph[index].clone())
        .filter(|name| global_names.contains(name.as_str()))
        .collect())
}

fn get_indirect_name(name: &str) -> String {
    format!("{}$indirect", name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types;

    #[test]
    fn sort_no_constants() {
        assert_eq!(
            sort_global_variables(&ir::Module::without_validation(vec![], vec![], vec![])),
            Ok(vec![])
        );
    }

    #[test]
    fn sort_constant() {
        assert_eq!(
            sort_global_variables(&ir::Module::without_validation(
                vec![],
                vec![ir::Definition::new("x", 42.0, types::Primitive::Float64).into()],
                vec![]
            )),
            Ok(vec!["x".into()])
        );
    }

    #[test]
    fn sort_sorted_constants() {
        assert_eq!(
            sort_global_variables(&ir::Module::without_validation(
                vec![],
                vec![
                    ir::Definition::new("x", 42.0, types::Primitive::Float64).into(),
                    ir::Definition::new("y", ir::Variable::new("x"), types::Primitive::Float64)
                        .into()
                ],
                vec![]
            )),
            Ok(vec!["x".into(), "y".into()])
        );
    }

    #[test]
    fn sort_constants_not_sorted() {
        assert_eq!(
            sort_global_variables(&ir::Module::without_validation(
                vec![],
                vec![
                    ir::Definition::new("y", ir::Variable::new("x"), types::Primitive::Float64)
                        .into(),
                    ir::Definition::new("x", 42.0, types::Primitive::Float64).into(),
                ],
                vec![]
            )),
            Ok(vec!["x".into(), "y".into()])
        );
    }

    #[test]
    fn sort_constants_not_sorted_with_function() {
        assert_eq!(
            sort_global_variables(&ir::Module::without_validation(
                vec![],
                vec![
                    ir::Definition::new(
                        "y",
                        ir::FunctionApplication::new(ir::Variable::new("f"), 42.0),
                        types::Primitive::Float64
                    ),
                    ir::Definition::new(
                        "f",
                        ir::Lambda::new(
                            vec![ir::Argument::new("a", types::Primitive::Float64)],
                            ir::Variable::new("x"),
                            types::Primitive::Float64
                        ),
                        types::Function::new(types::Primitive::Float64, types::Primitive::Float64)
                    ),
                    ir::Definition::new("x", 42.0, types::Primitive::Float64),
                ],
                vec![]
            )),
            Ok(vec!["x".into(), "f".into(), "y".into()])
        );
    }

    #[test]
    fn sort_constants_not_sorted_with_recursive_functions() {
        assert_eq!(
            sort_global_variables(&ir::Module::without_validation(
                vec![],
                vec![
                    ir::Definition::new(
                        "y",
                        ir::FunctionApplication::new(ir::Variable::new("f"), 42.0),
                        types::Primitive::Float64
                    )
                    .into(),
                    ir::Definition::new(
                        "f",
                        ir::Lambda::new(
                            vec![ir::Argument::new("a", types::Primitive::Float64)],
                            ir::FunctionApplication::new(
                                ir::Variable::new("g"),
                                ir::Variable::new("x")
                            ),
                            types::Primitive::Float64
                        ),
                        types::Function::new(types::Primitive::Float64, types::Primitive::Float64)
                    )
                    .into(),
                    ir::Definition::new(
                        "g",
                        ir::Lambda::new(
                            vec![ir::Argument::new("a", types::Primitive::Float64)],
                            ir::FunctionApplication::new(
                                ir::Variable::new("f"),
                                ir::Variable::new("x")
                            ),
                            types::Primitive::Float64
                        ),
                        types::Function::new(types::Primitive::Float64, types::Primitive::Float64)
                    )
                    .into(),
                    ir::Definition::new("x", 42.0, types::Primitive::Float64).into(),
                ],
                vec![]
            )),
            Ok(vec!["x".into(), "g".into(), "f".into(), "y".into()])
        );
    }

    #[test]
    fn fail_to_sort_recursively_defined_constant() {
        assert_eq!(
            sort_global_variables(&ir::Module::without_validation(
                vec![],
                vec![
                    ir::Definition::new("x", ir::Variable::new("x"), types::Primitive::Float64)
                        .into()
                ],
                vec![]
            )),
            Err(AnalysisError::CircularInitialization)
        );
    }

    #[test]
    fn fail_to_sort_recursively_defined_constants() {
        assert_eq!(
            sort_global_variables(&ir::Module::without_validation(
                vec![],
                vec![
                    ir::Definition::new("x", ir::Variable::new("y"), types::Primitive::Float64)
                        .into(),
                    ir::Definition::new("y", ir::Variable::new("x"), types::Primitive::Float64)
                        .into(),
                ],
                vec![]
            )),
            Err(AnalysisError::CircularInitialization)
        );
    }

    #[test]
    fn fail_to_sort_constant_recursive_through_function() {
        assert_eq!(
            sort_global_variables(&ir::Module::without_validation(
                vec![],
                vec![
                    ir::Definition::new(
                        "x",
                        ir::FunctionApplication::new(ir::Variable::new("f"), 42.0),
                        types::Primitive::Float64
                    ),
                    ir::Definition::new(
                        "f",
                        ir::Lambda::new(
                            vec![ir::Argument::new("a", types::Primitive::Float64)],
                            ir::Variable::new("x"),
                            types::Primitive::Float64
                        ),
                        types::Function::new(types::Primitive::Float64, types::Primitive::Float64)
                    ),
                ],
                vec![]
            )),
            Err(AnalysisError::CircularInitialization)
        );
    }
}
