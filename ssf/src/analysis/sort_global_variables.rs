use super::error::AnalysisError;
use crate::ast;
use petgraph::algo::toposort;
use petgraph::graph::Graph;
use std::collections::{HashMap, HashSet};

pub fn sort_global_variables(module: &ast::Module) -> Result<Vec<&str>, AnalysisError> {
    let value_names = module
        .definitions()
        .iter()
        .map(|definition| match definition {
            ast::Definition::FunctionDefinition(_) => None,
            ast::Definition::ValueDefinition(value_definition) => Some(value_definition.name()),
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
            sort_global_variables(&ast::Module::without_validation(vec![], vec![], vec![])),
            Ok(vec![])
        );
    }

    #[test]
    fn sort_a_constant() {
        assert_eq!(
            sort_global_variables(&ast::Module::without_validation(
                vec![],
                vec![ast::ValueDefinition::new("x", 42.0, types::Value::Number).into()],
                vec![]
            )),
            Ok(vec!["x".into()])
        );
    }

    #[test]
    fn sort_sorted_constants() {
        assert_eq!(
            sort_global_variables(&ast::Module::without_validation(
                vec![],
                vec![
                    ast::ValueDefinition::new("x", 42.0, types::Value::Number).into(),
                    ast::ValueDefinition::new("y", ast::Variable::new("x"), types::Value::Number)
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
            sort_global_variables(&ast::Module::without_validation(
                vec![],
                vec![
                    ast::ValueDefinition::new("y", ast::Variable::new("x"), types::Value::Number)
                        .into(),
                    ast::ValueDefinition::new("x", 42.0, types::Value::Number).into(),
                ],
                vec![]
            )),
            Ok(vec!["x", "y"])
        );
    }

    #[test]
    fn sort_constants_not_sorted_with_function() {
        assert_eq!(
            sort_global_variables(&ast::Module::without_validation(
                vec![],
                vec![
                    ast::ValueDefinition::new(
                        "y",
                        ast::Application::new(
                            ast::Variable::new("f"),
                            vec![ast::Expression::Number(42.0)]
                        ),
                        types::Value::Number
                    )
                    .into(),
                    ast::FunctionDefinition::new(
                        "f",
                        vec![ast::Argument::new("a", types::Value::Number)],
                        ast::Variable::new("x"),
                        types::Value::Number
                    )
                    .into(),
                    ast::ValueDefinition::new("x", 42.0, types::Value::Number).into(),
                ],
                vec![]
            )),
            Ok(vec!["x", "y"])
        );
    }

    #[test]
    fn sort_constants_not_sorted_with_recursive_functions() {
        assert_eq!(
            sort_global_variables(&ast::Module::without_validation(
                vec![],
                vec![
                    ast::ValueDefinition::new(
                        "y",
                        ast::Application::new(
                            ast::Variable::new("f"),
                            vec![ast::Expression::Number(42.0)]
                        ),
                        types::Value::Number
                    )
                    .into(),
                    ast::FunctionDefinition::new(
                        "f",
                        vec![ast::Argument::new("a", types::Value::Number)],
                        ast::Application::new(
                            ast::Variable::new("g"),
                            vec![ast::Variable::new("x").into()]
                        ),
                        types::Value::Number
                    )
                    .into(),
                    ast::FunctionDefinition::new(
                        "g",
                        vec![ast::Argument::new("a", types::Value::Number)],
                        ast::Application::new(
                            ast::Variable::new("f"),
                            vec![ast::Variable::new("x").into()]
                        ),
                        types::Value::Number
                    )
                    .into(),
                    ast::ValueDefinition::new("x", 42.0, types::Value::Number).into(),
                ],
                vec![]
            )),
            Ok(vec!["x", "y"])
        );
    }

    #[test]
    fn fail_to_sort_recursively_defined_constant() {
        assert_eq!(
            sort_global_variables(&ast::Module::without_validation(
                vec![],
                vec![
                    ast::ValueDefinition::new("x", ast::Variable::new("x"), types::Value::Number)
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
            sort_global_variables(&ast::Module::without_validation(
                vec![],
                vec![
                    ast::ValueDefinition::new("x", ast::Variable::new("y"), types::Value::Number)
                        .into(),
                    ast::ValueDefinition::new("y", ast::Variable::new("x"), types::Value::Number)
                        .into(),
                ],
                vec![]
            )),
            Err(AnalysisError::CircularInitialization)
        );
    }
}
