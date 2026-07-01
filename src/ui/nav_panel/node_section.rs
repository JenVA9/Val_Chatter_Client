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
    collapsed: &mut HashSet<String>,
) {
    if nodes.is_empty() { return; }

    let is_collapsed = collapsed.contains(label);

    ui.add_space(6.0);

    // Clickable header toggles collapse
    let arrow = if is_collapsed { "[+]" } else { "[-]" };
    let hdr = ui.add(
        egui::Label::new(
            egui::RichText::new(format!("{} {}", arrow, label)).small().strong()
        ).sense(egui::Sense::click())
    );
    if hdr.clicked() {
        if is_collapsed { collapsed.remove(label); } else { collapsed.insert(label.to_string()); }
    }

    if is_collapsed { return; }

    ui.add_space(2.0);
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
