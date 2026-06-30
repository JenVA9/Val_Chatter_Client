use crate::models::Node;
use super::node_button;

pub fn show(ui: &mut egui::Ui, maps: &[&Node], selected: &mut Option<i64>) {
    ui.horizontal_wrapped(|ui| {
        for map in maps {
            let is_sel = *selected == Some(map.id);
            if node_button::show(ui, &map.name, is_sel, egui::Color32::from_rgb(70, 130, 180)) {
                *selected = if is_sel { None } else { Some(map.id) };
            }
        }
    });
}
