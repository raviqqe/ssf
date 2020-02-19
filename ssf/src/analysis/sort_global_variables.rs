use super::error::AnalysisError;
use crate::ir;
use petgraph::algo::toposort;
use petgraph::graph::Graph;
use std::collections::{HashMap, HashSet};

pub fn sort_global_variables(module: &ir::Module) -> Result<Vec<&str>, AnalysisError> {
    let value_names = module
        .definitions()
        .iter()
        .map(|definition| match definition {
            ir::Definition::FunctionDefinition(_) => None,
            ir::Definition::ValueDefinition(value_definition) => Some(value_definition.name()),
        })
        .filter(|option| option.is_some())
        .collect::<Option<HashSet<&str>>>()
        .unwrap_or_else(HashSet::new);

    let mut graph = Graph::<&str, ()>::new();
    let mut name_indices = HashMap::<&str, _>::new();

    for definition in module.definitions() {
        name_indices.insert(definition.name(), graph.add_node(definition.name()));
    }

    for definition in module.definitions() {
        for name in definition.find_variables(&HashSet::new()) {
            if value_names.contains(name.as_str()) {
                graph.add_edge(
                    name_indices[name.as_str()],
                    name_indices[definition.name()],
                    (),
                );
            }
        }
    }

    Ok(toposort(&graph, None)?
        .into_iter()
        .map(|index| graph[index])
        .filter(|name| value_names.contains(name))
        .collect())
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
    fn sort_a_constant() {
        assert_eq!(
            sort_global_variables(&ir::Module::without_validation(
                vec![],
                vec![ir::ValueDefinition::new("x", 42.0, types::Value::Number).into()],
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
                    ir::ValueDefinition::new("x", 42.0, types::Value::Number).into(),
                    ir::ValueDefinition::new("y", ir::Variable::new("x"), types::Value::Number)
                        .into()
                ],
                vec![]
            )),
            Ok(vec!["x", "y"])
        );
    }

    #[test]
    fn sort_constants_not_sorted() {
        assert_eq!(
            sort_global_variables(&ir::Module::without_validation(
                vec![],
                vec![
                    ir::ValueDefinition::new("y", ir::Variable::new("x"), types::Value::Number)
                        .into(),
                    ir::ValueDefinition::new("x", 42.0, types::Value::Number).into(),
                ],
                vec![]
            )),
            Ok(vec!["x", "y"])
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
                        ir::FunctionApplication::new(
                            ir::Variable::new("f"),
                            vec![ir::Expression::Number(42.0)]
                        ),
                        types::Value::Number
                    )
                    .into(),
                    ir::FunctionDefinition::new(
                        "f",
                        vec![ir::Argument::new("a", types::Value::Number)],
                        ir::Variable::new("x"),
                        types::Value::Number
                    )
                    .into(),
                    ir::ValueDefinition::new("x", 42.0, types::Value::Number).into(),
                ],
                vec![]
            )),
            Ok(vec!["x", "y"])
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
                        ir::FunctionApplication::new(
                            ir::Variable::new("f"),
                            vec![ir::Expression::Number(42.0)]
                        ),
                        types::Value::Number
                    )
                    .into(),
                    ir::FunctionDefinition::new(
                        "f",
                        vec![ir::Argument::new("a", types::Value::Number)],
                        ir::FunctionApplication::new(
                            ir::Variable::new("g"),
                            vec![ir::Variable::new("x").into()]
                        ),
                        types::Value::Number
                    )
                    .into(),
                    ir::FunctionDefinition::new(
                        "g",
                        vec![ir::Argument::new("a", types::Value::Number)],
                        ir::FunctionApplication::new(
                            ir::Variable::new("f"),
                            vec![ir::Variable::new("x").into()]
                        ),
                        types::Value::Number
                    )
                    .into(),
                    ir::ValueDefinition::new("x", 42.0, types::Value::Number).into(),
                ],
                vec![]
            )),
            Ok(vec!["x", "y"])
        );
    }

    #[test]
    fn fail_to_sort_recursively_defined_constant() {
        assert_eq!(
            sort_global_variables(&ir::Module::without_validation(
                vec![],
                vec![
                    ir::ValueDefinition::new("x", ir::Variable::new("x"), types::Value::Number)
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
                    ir::ValueDefinition::new("x", ir::Variable::new("y"), types::Value::Number)
                        .into(),
                    ir::ValueDefinition::new("y", ir::Variable::new("x"), types::Value::Number)
                        .into(),
                ],
                vec![]
            )),
            Err(AnalysisError::CircularInitialization)
        );
    }
}
