/// Draws a small rounded tag/chip label.
pub fn show(ui: &mut egui::Ui, label: &str, color: egui::Color32) {
    let text = egui::RichText::new(label).small().color(egui::Color32::WHITE);
    let frame = egui::Frame::default()
        .fill(color)
        .inner_margin(egui::Margin::symmetric(6.0, 2.0))
        .rounding(4.0);
    frame.show(ui, |ui| { ui.label(text); });
}
