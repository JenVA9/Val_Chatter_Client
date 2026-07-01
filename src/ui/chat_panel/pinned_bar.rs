use crate::models::Message;

pub struct PinnedBarAction {
    pub jump_to: Option<i64>,
}

pub fn show(ui: &mut egui::Ui, pinned: &[Message]) -> PinnedBarAction {
    let mut action = PinnedBarAction { jump_to: None };

    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Pinned:").small().strong());

        for msg in pinned.iter().take(3) {
            ui.separator();

            let pin_marker = if let Some(exp) = &msg.pin_expires_at {
                format!("[{}] ", pin_time_short(exp))
            } else {
                "∞ ".to_string()
            };

            let preview = msg.content.as_deref()
                .map(|s| if s.len() > 35 { format!("{}…", &s[..35]) } else { s.to_string() })
                .unwrap_or_else(|| "[image]".to_string());

            let label = format!("{}{}: {}", pin_marker, msg.username, preview);
            let resp = ui.add(
                egui::Label::new(egui::RichText::new(&label).small().weak())
                    .sense(egui::Sense::click()),
            );
            if resp.clicked() {
                action.jump_to = Some(msg.id);
            }
            resp.on_hover_text("Click to jump to message");
        }

        if pinned.len() > 3 {
            ui.label(egui::RichText::new(format!("(+{})", pinned.len() - 3)).small().weak());
        }
    });

    action
}

fn pin_time_short(expires_at: &str) -> String {
    if expires_at.len() >= 16 {
        expires_at[11..16].to_string()
    } else {
        "timed".to_string()
    }
}
