/// Returns true if the button was clicked.
/// Visual tiers:
///   selected + has_content  → neon fill (bright accent)
///   selected, no content    → normal accent fill
///   unselected + has_content→ neon outline (thick, bright stroke)
///   unselected, no content  → dim outline (thin, faded)
pub fn show(
    ui: &mut egui::Ui,
    label: &str,
    selected: bool,
    has_content: bool,
    accent: egui::Color32,
) -> bool {
    let neon = neon_of(accent);

    let (bg, text_color, stroke_color, stroke_w) = match (selected, has_content) {
        (true,  true)  => (neon,                         egui::Color32::WHITE,             neon,   2.0),
        (true,  false) => (accent,                        egui::Color32::WHITE,             accent, 1.0),
        (false, true)  => (egui::Color32::TRANSPARENT,   ui.visuals().text_color(),        neon,   2.0),
        (false, false) => (egui::Color32::TRANSPARENT,   ui.visuals().weak_text_color(),   accent, 1.0),
    };

    let button = egui::Button::new(egui::RichText::new(label).color(text_color))
        .fill(bg)
        .stroke(egui::Stroke::new(stroke_w, stroke_color))
        .rounding(4.0);

    ui.add(button).clicked()
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
