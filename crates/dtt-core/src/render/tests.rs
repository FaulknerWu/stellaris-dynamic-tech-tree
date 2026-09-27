use super::*;
use crate::test_support::{catalog, definition, ids};

fn graph(edges: &[(&str, &[&str])]) -> Graph {
    let catalog = catalog(edges.iter().map(|(id, prerequisites)| {
        let mut node = definition(id);
        node.prerequisites.all_of = ids(prerequisites);
        node
    }));
    Graph::build(&catalog, &catalog.sorted_ids())
}

fn render(graph: &Graph, limits: RenderLimits) -> RenderOutcome {
    render_tree(
        &"root".into(),
        &RenderInput {
            graph,
            limits,
            format_node: &|id, _| id.to_string(),
        },
    )
}

#[test]
fn diamond_graph_displays_shared_nodes_but_expands_descendants_once() {
    let graph = graph(&[
        ("root", &[]),
        ("a", &["root"]),
        ("b", &["root"]),
        ("c", &["a", "b"]),
        ("d", &["c"]),
    ]);
    assert_eq!(
        render(&graph, RenderLimits::default()),
        RenderOutcome::Tree("|-a\n|   |-c\n|       |-d\n|-b\n    |-c".into())
    );
}

#[test]
fn cycles_are_reported_and_rendering_terminates() {
    let graph = graph(&[("root", &["root", "b"]), ("a", &["root"]), ("b", &["a"])]);
    assert_eq!(graph.cycle_report().self_references, ids(&["root"]));
    assert_eq!(
        graph.cycle_report().complex_cycles,
        vec![ids(&["a", "b", "root"])]
    );
    assert_eq!(
        render(&graph, RenderLimits::default()),
        RenderOutcome::Tree("|-a\n|   |-b\n|       |-root\n|-root".into())
    );
}

#[test]
fn depth_and_node_budgets_bound_rendering_including_zero() {
    let graph = graph(&[("root", &[]), ("a", &["root"]), ("b", &["a"])]);
    for (max_depth, max_total_nodes, expected) in [
        (0, 10, ""),
        (1, 10, "|-a"),
        (10, 0, "|-..."),
        (10, 1, "|-a\n    |-..."),
    ] {
        assert_eq!(
            render(
                &graph,
                RenderLimits {
                    max_depth,
                    max_total_nodes,
                    ..Default::default()
                }
            ),
            RenderOutcome::Tree(expected.into())
        );
    }
}

#[test]
fn sibling_limit_and_overlong_threshold_have_distinct_boundaries() {
    let graph = graph(&[
        ("root", &[]),
        ("a", &["root"]),
        ("b", &["root"]),
        ("c", &["root"]),
    ]);
    let limits = RenderLimits {
        max_siblings: 2,
        overlong_root_threshold: 3,
        ..Default::default()
    };
    assert_eq!(
        render(&graph, limits.clone()),
        RenderOutcome::Tree("|-a\n    |-... (+2)".into())
    );
    assert_eq!(
        render(
            &graph,
            RenderLimits {
                overlong_root_threshold: 2,
                ..limits
            }
        ),
        RenderOutcome::OverlongRoot {
            root: "root".into(),
            child_count: 3,
            limit: 2
        }
    );
}

#[test]
fn additional_requirements_drop_the_satisfied_or_group_without_flattening_others() {
    let mut child = definition("child");
    child.prerequisites = Prerequisites {
        all_of: ids(&["required"]),
        any_of_groups: vec![ids(&["root", "alternative"]), ids(&["x", "y"])],
    };
    let catalog = catalog([definition("root"), child]);
    let graph = Graph::build(&catalog, &catalog.sorted_ids());
    let result = render_tree(
        &"root".into(),
        &RenderInput {
            graph: &graph,
            limits: Default::default(),
            format_node: &|id, requirements| {
                assert_eq!(requirements.all_of, ids(&["required"]));
                assert_eq!(requirements.any_of_groups, vec![ids(&["x", "y"])]);
                id.to_string()
            },
        },
    );
    assert_eq!(result, RenderOutcome::Tree("|-child".into()));
}
