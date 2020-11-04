use super::error::AnalysisError;
use crate::ir;
use petgraph::algo::kosaraju_scc;
use petgraph::graph::Graph;
use std::collections::{HashMap, HashSet};

pub fn sort_global_variables(module: &ir::Module) -> Result<Vec<String>, AnalysisError> {
    let mut graph = Graph::<String, ()>::new();
    let mut name_indices = HashMap::<String, _>::new();

    for definition in module.definitions() {
        name_indices.insert(
            definition.name().into(),
            graph.add_node(definition.name().into()),
        );
    }

    for definition in module.definitions() {
        for name in definition.find_variables() {
            if definition.name() == name {
                return Err(AnalysisError::CircularInitialization);
            }

            graph.add_edge(
                name_indices[name.as_str()],
                name_indices[definition.name()],
                (),
            );
        }
    }

    let value_names = module
        .definitions()
        .iter()
        .filter_map(|definition| {
            definition
                .to_value_definition()
                .map(|value_definition| value_definition.name())
        })
        .collect::<HashSet<_>>();

    Ok(kosaraju_scc(&graph)
        .into_iter()
        .map(|indices| {
            indices
                .into_iter()
                .map(|index| graph[index].clone())
                .collect::<Vec<String>>()
        })
        .filter(|names| names.iter().any(|name| value_names.contains(name.as_str())))
        .map(|mut names| {
            if names.len() > 1 {
                Err(AnalysisError::CircularInitialization)
            } else {
                Ok(names.drain(..).next().unwrap())
            }
        })
        .rev()
        .collect::<Result<_, _>>()?)
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
                vec![ir::ValueDefinition::new("x", 42.0, types::Primitive::Float64).into()],
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
                    ir::ValueDefinition::new("x", 42.0, types::Primitive::Float64).into(),
                    ir::ValueDefinition::new(
                        "y",
                        ir::Variable::new("x"),
                        types::Primitive::Float64
                    )
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
                    ir::ValueDefinition::new(
                        "y",
                        ir::Variable::new("x"),
                        types::Primitive::Float64
                    )
                    .into(),
                    ir::ValueDefinition::new("x", 42.0, types::Primitive::Float64).into(),
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
                    ir::ValueDefinition::new(
                        "y",
                        ir::FunctionApplication::new(ir::Variable::new("f"), vec![42.0.into()]),
                        types::Primitive::Float64
                    )
                    .into(),
                    ir::FunctionDefinition::new(
                        "f",
                        vec![ir::Argument::new("a", types::Primitive::Float64)],
                        ir::Variable::new("x"),
                        types::Primitive::Float64,
                    )
                    .into(),
                    ir::ValueDefinition::new("x", 42.0, types::Primitive::Float64).into(),
                ],
                vec![]
            )),
            Ok(vec!["x".into(), "y".into()])
        );
    }

    #[test]
    fn sort_constants_not_sorted_with_recursive_functions() {
        assert_eq!(
            sort_global_variables(&ir::Module::without_validation(
                vec![],
                vec![
                    ir::ValueDefinition::new(
                        "y",
                        ir::FunctionApplication::new(ir::Variable::new("f"), vec![42.0.into()]),
                        types::Primitive::Float64
                    )
                    .into(),
                    ir::FunctionDefinition::new(
                        "f",
                        vec![ir::Argument::new("a", types::Primitive::Float64)],
                        ir::FunctionApplication::new(
                            ir::Variable::new("g"),
                            vec![ir::Variable::new("x").into()]
                        ),
                        types::Primitive::Float64,
                    )
                    .into(),
                    ir::FunctionDefinition::new(
                        "g",
                        vec![ir::Argument::new("a", types::Primitive::Float64)],
                        ir::FunctionApplication::new(
                            ir::Variable::new("f"),
                            vec![ir::Variable::new("x").into()]
                        ),
                        types::Primitive::Float64,
                    )
                    .into(),
                    ir::ValueDefinition::new("x", 42.0, types::Primitive::Float64).into(),
                ],
                vec![]
            )),
            Ok(vec!["x".into(), "y".into()])
        );
    }

    #[test]
    fn fail_to_sort_recursively_defined_constant() {
        assert_eq!(
            sort_global_variables(&ir::Module::without_validation(
                vec![],
                vec![ir::ValueDefinition::new(
                    "x",
                    ir::Variable::new("x"),
                    types::Primitive::Float64
                )
                .into()],
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
                    ir::ValueDefinition::new(
                        "x",
                        ir::Variable::new("y"),
                        types::Primitive::Float64
                    )
                    .into(),
                    ir::ValueDefinition::new(
                        "y",
                        ir::Variable::new("x"),
                        types::Primitive::Float64
                    )
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
                    ir::ValueDefinition::new(
                        "x",
                        ir::FunctionApplication::new(ir::Variable::new("f"), vec![42.0.into()]),
                        types::Primitive::Float64
                    )
                    .into(),
                    ir::FunctionDefinition::new(
                        "f",
                        vec![ir::Argument::new("a", types::Primitive::Float64)],
                        ir::Variable::new("x"),
                        types::Primitive::Float64,
                    )
                    .into(),
                ],
                vec![]
            )),
            Err(AnalysisError::CircularInitialization)
        );
    }
}
