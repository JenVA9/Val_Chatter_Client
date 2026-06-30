use std::collections::HashSet;
use crate::models::Node;
use super::node_button;

pub fn show(
    ui: &mut egui::Ui,
    label: &str,
    nodes: &[&Node],
    selected: &mut Option<i64>,
    color: egui::Color32,
    active_nodes: &HashSet<i64>,
) {
    if nodes.is_empty() { return; }

    ui.add_space(8.0);
    ui.label(egui::RichText::new(label).small().strong());
    ui.add_space(4.0);

    ui.horizontal_wrapped(|ui| {
        for node in nodes {
            let is_selected = *selected == Some(node.id);
            let has_content = active_nodes.contains(&node.id);
            if node_button::show(ui, &node.name, is_selected, has_content, color) {
                *selected = if is_selected { None } else { Some(node.id) };
            }
        }
    });
}
