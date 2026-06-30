use crate::models::Message;

pub fn show(ui: &mut egui::Ui, pinned: &[Message]) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("📌 Pinned:").small().strong());
        for msg in pinned.iter().take(3) {
            ui.separator();
            let preview = msg.content.as_deref()
                .map(|s| if s.len() > 40 { format!("{}…", &s[..40]) } else { s.to_string() })
                .unwrap_or_else(|| "[image]".to_string());
            ui.label(
                egui::RichText::new(format!("{}: {}", msg.username, preview))
                    .small()
                    .weak(),
            );
        }
        if pinned.len() > 3 {
            ui.label(egui::RichText::new(format!("(+{})", pinned.len() - 3)).small().weak());
        }
    });
}
