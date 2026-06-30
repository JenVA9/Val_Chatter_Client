pub mod map_selector;
pub mod node_section;
pub mod node_button;

use std::collections::HashSet;
use crate::models::{Node, NodeType};
use crate::state::NavState;

pub fn show(
    ui: &mut egui::Ui,
    nodes: &[Node],
    nav: &mut NavState,
    active_nodes: &HashSet<i64>,
) -> bool {
    let before = nav.clone();

    ui.heading("Navigation");
    ui.separator();

    egui::ScrollArea::vertical().show(ui, |ui| {
        let maps: Vec<&Node> = nodes.iter().filter(|n| n.node_type == NodeType::Map).collect();
        node_section::show(ui, "Map", &maps, &mut nav.selected_map,
            egui::Color32::from_rgb(70, 130, 180), active_nodes);

        if let Some(map_id) = nav.selected_map {
            let sites: Vec<&Node> = nodes.iter()
                .filter(|n| n.node_type == NodeType::Site && n.parent_id == Some(map_id))
                .collect();
            if !sites.is_empty() {
                node_section::show(ui, "Site", &sites, &mut nav.selected_site,
                    egui::Color32::from_rgb(100, 160, 100), active_nodes);
            }
        } else {
            nav.selected_site = None;
        }

        let agents: Vec<&Node> = nodes.iter().filter(|n| n.node_type == NodeType::Agent).collect();
        node_section::show(ui, "Agent", &agents, &mut nav.selected_agent,
            egui::Color32::from_rgb(200, 80, 80), active_nodes);

        let types: Vec<&Node> = nodes.iter().filter(|n| n.node_type == NodeType::TacticType).collect();
        node_section::show(ui, "Tactic Type", &types, &mut nav.selected_type,
            egui::Color32::from_rgb(160, 100, 200), active_nodes);

        ui.add_space(12.0);
        ui.separator();
        ui.label(egui::RichText::new("Current Context").small().weak());
        ui.add_space(4.0);

        let context_label = build_context_label(nodes, nav);
        if context_label.is_empty() {
            ui.label(egui::RichText::new("Nothing selected").italics().weak());
        } else {
            ui.label(&context_label);
        }

        if !nav.is_empty() {
            ui.add_space(4.0);
            if ui.small_button("Clear selection").clicked() {
                *nav = NavState::default();
            }
        }
    });

    *nav != before
}

fn build_context_label(nodes: &[Node], nav: &NavState) -> String {
    let mut parts = Vec::new();
    let find = |id: i64| nodes.iter().find(|n| n.id == id).map(|n| n.name.as_str()).unwrap_or("?");
    if let Some(id) = nav.selected_map { parts.push(find(id).to_string()); }
    if let Some(id) = nav.selected_site { parts.push(find(id).to_string()); }
    if let Some(id) = nav.selected_agent { parts.push(find(id).to_string()); }
    if let Some(id) = nav.selected_type { parts.push(find(id).to_string()); }
    parts.join(" + ")
}
