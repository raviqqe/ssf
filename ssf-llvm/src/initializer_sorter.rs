use super::error::CompileError;
use petgraph::algo::toposort;
use petgraph::graph::Graph;
use std::collections::{HashMap, HashSet};

pub struct InitializerSorter;

impl InitializerSorter {
    pub fn sort(module: &ssf::ast::Module) -> Result<Vec<&str>, CompileError> {
        let value_names = module
            .definitions()
            .iter()
            .map(|definition| match definition {
                ssf::ast::Definition::FunctionDefinition(_) => None,
                ssf::ast::Definition::ValueDefinition(value_definition) => {
                    Some(value_definition.name())
                }
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
            for name in definition.find_global_variables(&HashSet::new()) {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sort_no_constants() {
        assert_eq!(
            InitializerSorter::sort(&ssf::ast::Module::new(vec![], vec![])),
            Ok(vec![])
        );
    }

    #[test]
    fn sort_a_constant() {
        assert_eq!(
            InitializerSorter::sort(&ssf::ast::Module::new(
                vec![],
                vec![ssf::ast::ValueDefinition::new("x", 42.0, ssf::types::Value::Number).into()]
            )),
            Ok(vec!["x"])
        );
    }

    #[test]
    fn sort_sorted_constants() {
        assert_eq!(
            InitializerSorter::sort(&ssf::ast::Module::new(
                vec![],
                vec![
                    ssf::ast::ValueDefinition::new("x", 42.0, ssf::types::Value::Number).into(),
                    ssf::ast::ValueDefinition::new(
                        "y",
                        ssf::ast::Variable::new("x"),
                        ssf::types::Value::Number
                    )
                    .into()
                ]
            )),
            Ok(vec!["x", "y"])
        );
    }

    #[test]
    fn sort_constants_not_sorted() {
        assert_eq!(
            InitializerSorter::sort(&ssf::ast::Module::new(
                vec![],
                vec![
                    ssf::ast::ValueDefinition::new(
                        "y",
                        ssf::ast::Variable::new("x"),
                        ssf::types::Value::Number
                    )
                    .into(),
                    ssf::ast::ValueDefinition::new("x", 42.0, ssf::types::Value::Number).into(),
                ]
            )),
            Ok(vec!["x", "y"])
        );
    }

    #[test]
    fn sort_constants_not_sorted_with_function() {
        assert_eq!(
            InitializerSorter::sort(&ssf::ast::Module::new(
                vec![],
                vec![
                    ssf::ast::ValueDefinition::new(
                        "y",
                        ssf::ast::Application::new(
                            ssf::ast::Variable::new("f"),
                            vec![ssf::ast::Expression::Number(42.0)]
                        ),
                        ssf::types::Value::Number
                    )
                    .into(),
                    ssf::ast::FunctionDefinition::new(
                        "f",
                        vec![],
                        vec![ssf::ast::Argument::new("a", ssf::types::Value::Number)],
                        ssf::ast::Variable::new("x"),
                        ssf::types::Value::Number
                    )
                    .into(),
                    ssf::ast::ValueDefinition::new("x", 42.0, ssf::types::Value::Number).into(),
                ]
            )),
            Ok(vec!["x", "y"])
        );
    }

    #[test]
    fn sort_constants_not_sorted_with_recursive_functions() {
        assert_eq!(
            InitializerSorter::sort(&ssf::ast::Module::new(
                vec![],
                vec![
                    ssf::ast::ValueDefinition::new(
                        "y",
                        ssf::ast::Application::new(
                            ssf::ast::Variable::new("f"),
                            vec![ssf::ast::Expression::Number(42.0)]
                        ),
                        ssf::types::Value::Number
                    )
                    .into(),
                    ssf::ast::FunctionDefinition::new(
                        "f",
                        vec![],
                        vec![ssf::ast::Argument::new("a", ssf::types::Value::Number)],
                        ssf::ast::Application::new(
                            ssf::ast::Variable::new("g"),
                            vec![ssf::ast::Variable::new("x").into()]
                        ),
                        ssf::types::Value::Number
                    )
                    .into(),
                    ssf::ast::FunctionDefinition::new(
                        "g",
                        vec![],
                        vec![ssf::ast::Argument::new("a", ssf::types::Value::Number)],
                        ssf::ast::Application::new(
                            ssf::ast::Variable::new("f"),
                            vec![ssf::ast::Variable::new("x").into()]
                        ),
                        ssf::types::Value::Number
                    )
                    .into(),
                    ssf::ast::ValueDefinition::new("x", 42.0, ssf::types::Value::Number).into(),
                ]
            )),
            Ok(vec!["x", "y"])
        );
    }

    #[test]
    fn fail_to_sort_recursively_defined_constant() {
        assert_eq!(
            InitializerSorter::sort(&ssf::ast::Module::new(
                vec![],
                vec![ssf::ast::ValueDefinition::new(
                    "x",
                    ssf::ast::Variable::new("x"),
                    ssf::types::Value::Number
                )
                .into()]
            )),
            Err(CompileError::CircularInitialization)
        );
    }

    #[test]
    fn fail_to_sort_recursively_defined_constants() {
        assert_eq!(
            InitializerSorter::sort(&ssf::ast::Module::new(
                vec![],
                vec![
                    ssf::ast::ValueDefinition::new(
                        "x",
                        ssf::ast::Variable::new("y"),
                        ssf::types::Value::Number
                    )
                    .into(),
                    ssf::ast::ValueDefinition::new(
                        "y",
                        ssf::ast::Variable::new("x"),
                        ssf::types::Value::Number
                    )
                    .into(),
                ]
            )),
            Err(CompileError::CircularInitialization)
        );
    }
}
