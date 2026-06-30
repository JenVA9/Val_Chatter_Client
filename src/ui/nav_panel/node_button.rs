/// Returns true if the button was clicked.
pub fn show(ui: &mut egui::Ui, label: &str, selected: bool, accent: egui::Color32) -> bool {
    let (bg, text_color) = if selected {
        (accent, egui::Color32::WHITE)
    } else {
        (egui::Color32::TRANSPARENT, ui.visuals().text_color())
    };

    let button = egui::Button::new(egui::RichText::new(label).color(text_color))
        .fill(bg)
        .stroke(egui::Stroke::new(1.0, accent))
        .rounding(4.0);

    ui.add(button).clicked()
}
