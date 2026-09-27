use dtt_core::graph::Graph;
use dtt_core::render::{RenderInput, RenderLimits, RenderOutcome, render_tree};
use dtt_core::technology::{Area, Catalog, Definition, Id, Prerequisites, SwapResolution};

use dtt_i18n::{TranslationError, Translator};
use std::cell::RefCell;

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
    translator: &Translator,
) -> Result<String, TranslationError> {
    let failure = RefCell::new(None);
    let format_node = |technology_id: &Id, additional: &Prerequisites| {
        format_node_line(technology_id, additional, technologies, swaps, translator).unwrap_or_else(
            |error| {
                *failure.borrow_mut() = Some(error);
                String::new()
            },
        )
    };
    let render_input = RenderInput {
        graph,
        limits: limits.clone(),
        format_node: &format_node,
    };
    let body = match render_tree(root, &render_input) {
        RenderOutcome::Tree(body) => body,
        RenderOutcome::OverlongRoot {
            root,
            child_count,
            limit,
        } => translator.tree_omitted(root.as_str(), child_count, limit)?,
    };
    if let Some(error) = failure.into_inner() {
        return Err(error);
    }
    Ok(format_tree_content(&body))
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
    translator: &Translator,
) -> Result<String, TranslationError> {
    let mut line = format_single_technology(technology_id, technologies, swaps);
    let mut additional: Vec<String> = additional_prerequisites
        .all_of
        .iter()
        .filter(|prerequisite| technologies.contains(prerequisite))
        .map(|prerequisite| format_single_technology(prerequisite, technologies, swaps))
        .collect();
    for group in &additional_prerequisites.any_of_groups {
        let mut alternatives = group
            .iter()
            .filter(|id| technologies.contains(id))
            .map(|id| format_single_technology(id, technologies, swaps));
        if let Some(mut combined) = alternatives.next() {
            for alternative in alternatives {
                combined = translator.game_or(&combined, &alternative)?;
            }
            additional.push(format!("({combined})"));
        }
    }

    if !additional.is_empty() {
        line.push(' ');
        line.push_str(&translator.game_requires(&additional.join(" , "))?);
    }
    Ok(line)
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
