use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::graph::Graph;
use crate::technology::{Id, Prerequisites};

const TREE_BAR: &str = "|   ";
const TREE_EMPTY: &str = "    ";
const TREE_BRANCH: &str = "|-";
const ELLIPSIS: &str = "...";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderLimits {
    pub max_depth: usize,
    pub max_siblings: usize,
    pub max_total_nodes: usize,
    pub overlong_root_threshold: usize,
}

impl Default for RenderLimits {
    fn default() -> Self {
        RenderLimits {
            max_depth: 16,
            max_siblings: 64,
            max_total_nodes: 4096,
            overlong_root_threshold: 128,
        }
    }
}

pub struct RenderInput<'a> {
    pub graph: &'a Graph,
    pub limits: RenderLimits,
    pub format_node: &'a dyn Fn(&Id, &Prerequisites) -> String,
}

pub fn render_tree(root: &Id, input: &RenderInput) -> String {
    let direct = input.graph.unlocks(root);

    if direct.len() > input.limits.overlong_root_threshold {
        return format_overlong_root(root, direct.len(), input.limits.overlong_root_threshold);
    }

    let mut lines: Vec<String> = Vec::new();
    let mut remaining = input.limits.max_total_nodes;
    let mut visited = BTreeSet::from([root.clone()]);

    render_children(root, "", input, &mut visited, &mut remaining, &mut lines, 1);

    lines.join("\n")
}

fn format_overlong_root(root: &Id, n: usize, limit: usize) -> String {
    format!(
        "! overlong root: {root} has {n} direct children (limit {limit}) - tree omitted",
        root = root.as_str()
    )
}

fn render_children(
    parent: &Id,
    parent_prefix: &str,
    input: &RenderInput,
    visited: &mut BTreeSet<Id>,
    remaining: &mut usize,
    lines: &mut Vec<String>,
    depth: usize,
) {
    let children = input.graph.unlocks(parent);
    if children.is_empty() {
        return;
    }

    if depth > input.limits.max_depth {
        return;
    }

    let n = children.len();

    let (visible, truncated_extras) = if n > input.limits.max_siblings {
        let shown = input.limits.max_siblings.saturating_sub(1).max(1);
        (shown, Some(n - shown))
    } else {
        (n, None)
    };

    for (i, id) in children.iter().enumerate().take(visible) {
        let is_last_visible = i == visible - 1 && truncated_extras.is_none();

        if *remaining == 0 {
            lines.push(format!("{parent_prefix}{TREE_BRANCH}{ELLIPSIS}"));
            return;
        }
        *remaining -= 1;

        let additional_prerequisites = input
            .graph
            .prerequisites(id)
            .map(|requirements| requirements.remaining_after(parent))
            .unwrap_or_default();
        let is_repeated = visited.contains(id);

        lines.push(format!(
            "{}{}{}",
            parent_prefix,
            TREE_BRANCH,
            (input.format_node)(id, &additional_prerequisites)
        ));

        if !is_repeated {
            visited.insert(id.clone());
            let sub_prefix = format!(
                "{}{}",
                parent_prefix,
                if is_last_visible {
                    TREE_EMPTY
                } else {
                    TREE_BAR
                }
            );
            render_children(id, &sub_prefix, input, visited, remaining, lines, depth + 1);
        }
    }

    if let Some(extra) = truncated_extras {
        let trunc_prefix = format!("{parent_prefix}{TREE_EMPTY}");
        lines.push(format!(
            "{trunc_prefix}{TREE_BRANCH}{ELLIPSIS} {extra} more"
        ));
    }
}
