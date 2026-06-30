use std::collections::HashSet;
use crate::models::Node;
use super::node_button;

pub fn show(ui: &mut egui::Ui, maps: &[&Node], selected: &mut Option<i64>, active_nodes: &HashSet<i64>) {
    ui.horizontal_wrapped(|ui| {
        for map in maps {
            let is_sel = *selected == Some(map.id);
            let has_content = active_nodes.contains(&map.id);
            if node_button::show(ui, &map.name, is_sel, has_content, egui::Color32::from_rgb(70, 130, 180)) {
                *selected = if is_sel { None } else { Some(map.id) };
            }
        }
    });
}
