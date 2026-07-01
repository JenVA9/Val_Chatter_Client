/// Visual tiers:
///   selected + has_content  → neon fill + "●" prefix + thick stroke   (in this thread, has messages)
///   selected, no content    → accent fill, white text                 (selected, no messages yet)
///   unselected + has_content→ transparent, neon-colored text + border (has messages elsewhere)
///   unselected, no content  → transparent, dim text, hairline border  (empty)
pub fn show(
    ui: &mut egui::Ui,
    label: &str,
    selected: bool,
    has_content: bool,
    accent: egui::Color32,
) -> bool {
    let neon = neon_of(accent);

    match (selected, has_content) {
        (true, true) => {
            let display = format!("● {}", label);
            ui.add(
                egui::Button::new(egui::RichText::new(&display).color(egui::Color32::WHITE).strong())
                    .fill(neon)
                    .stroke(egui::Stroke::new(2.5, egui::Color32::WHITE.linear_multiply(0.5)))
                    .rounding(4.0),
            ).clicked()
        }
        (true, false) => {
            ui.add(
                egui::Button::new(egui::RichText::new(label).color(egui::Color32::WHITE))
                    .fill(accent)
                    .stroke(egui::Stroke::new(1.0, accent))
                    .rounding(4.0),
            ).clicked()
        }
        (false, true) => {
            let dim_neon = egui::Color32::from_rgba_unmultiplied(neon.r(), neon.g(), neon.b(), 200);
            ui.add(
                egui::Button::new(egui::RichText::new(label).color(dim_neon))
                    .fill(egui::Color32::TRANSPARENT)
                    .stroke(egui::Stroke::new(1.5, dim_neon))
                    .rounding(4.0),
            ).clicked()
        }
        (false, false) => {
            ui.add(
                egui::Button::new(
                    egui::RichText::new(label).color(ui.visuals().weak_text_color()),
                )
                    .fill(egui::Color32::TRANSPARENT)
                    .stroke(egui::Stroke::new(0.5, ui.visuals().weak_text_color()))
                    .rounding(4.0),
            ).clicked()
        }
    }
}

fn neon_of(accent: egui::Color32) -> egui::Color32 {
    let [r, g, b, a] = accent.to_array();
    let max = r.max(g).max(b);
    if max == 0 { return egui::Color32::from_rgba_unmultiplied(0, 200, 255, a); }
    let scale = 255.0 / max as f32;
    let nr = (r as f32 * scale).min(255.0) as u8;
    let ng = (g as f32 * scale).min(255.0) as u8;
    let nb = (b as f32 * scale).min(255.0) as u8;
    egui::Color32::from_rgba_unmultiplied(nr, ng, nb, a)
}
