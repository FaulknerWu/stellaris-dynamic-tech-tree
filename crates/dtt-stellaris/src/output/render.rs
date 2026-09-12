use dtt_core::graph::Graph;
use dtt_core::render::{RenderInput, RenderLimits, render_tree};
use dtt_core::technology::{Area, Catalog, Definition, Id, Prerequisites, SwapResolution};

use super::SupportedLanguage;

const TREE_HEADER: &str = "\\n\\n§H$technology_tree_title$§!";
const TREE_EMPTY_TRAILER: &str = "§Y$tech_tree_max_level$§!";
const DANGEROUS_TECH_COLOR: &str = "§R";
const RARE_TECH_COLOR: &str = "§M";
const NORMAL_TECH_COLOR: &str = "§W";

pub fn render_tree_content(
    root: &Id,
    graph: &Graph,
    technologies: &Catalog,
    swaps: &SwapResolution,
    limits: &RenderLimits,
    language: SupportedLanguage,
) -> String {
    let requires_text = language.strings().requires;
    let format_node = |technology_id: &Id, additional: &Prerequisites| {
        format_node_line(
            technology_id,
            additional,
            technologies,
            swaps,
            requires_text,
        )
    };
    let render_input = RenderInput {
        graph,
        limits: limits.clone(),
        format_node: &format_node,
    };
    format_tree_content(&render_tree(root, &render_input))
}

fn format_tree_content(body: &str) -> String {
    let content = if body.is_empty() {
        format!("{TREE_HEADER}\n{TREE_EMPTY_TRAILER}")
    } else {
        format!("{TREE_HEADER}\n{body}")
    };

    content.replace('\n', "\\n")
}

fn format_node_line(
    technology_id: &Id,
    additional_prerequisites: &Prerequisites,
    technologies: &Catalog,
    swaps: &SwapResolution,
    requires_text: &str,
) -> String {
    let mut line = format_single_technology(technology_id, technologies, swaps);
    let mut additional: Vec<String> = additional_prerequisites
        .all_of
        .iter()
        .filter(|prerequisite| technologies.contains(prerequisite))
        .map(|prerequisite| format_single_technology(prerequisite, technologies, swaps))
        .collect();
    additional.extend(
        additional_prerequisites
            .any_of_groups
            .iter()
            .filter_map(|group| {
                let alternatives: Vec<String> = group
                    .iter()
                    .filter(|prerequisite| technologies.contains(prerequisite))
                    .map(|prerequisite| format_single_technology(prerequisite, technologies, swaps))
                    .collect();
                (!alternatives.is_empty()).then(|| format!("({})", alternatives.join(" OR ")))
            }),
    );

    if !additional.is_empty() {
        line.push_str(&format!(
            " [§R{requires_text}§! {}]",
            additional.join(" , ")
        ));
    }
    line
}

fn format_single_technology(
    technology_id: &Id,
    technologies: &Catalog,
    swaps: &SwapResolution,
) -> String {
    let technology = technologies
        .get(technology_id)
        .expect("rendered technology must exist in the catalog");
    let display_id = swaps.display_of(technology_id);
    let area = swaps
        .display_area
        .get(technology_id)
        .copied()
        .unwrap_or(technology.area);
    format!(
        "({tier})['technology:{technology_id}', {area_icon}{color}${display_id}$§!]",
        tier = technology.tier,
        technology_id = technology_id.as_str(),
        area_icon = area_icon(area),
        color = technology_color(technology),
    )
}

fn technology_color(technology: &Definition) -> &'static str {
    if technology.is_dangerous {
        DANGEROUS_TECH_COLOR
    } else if technology.is_rare {
        RARE_TECH_COLOR
    } else {
        NORMAL_TECH_COLOR
    }
}

fn area_icon(area: Area) -> &'static str {
    match area {
        Area::Physics => "£physics£",
        Area::Society => "£society£",
        Area::Engineering => "£engineering£",
    }
}
