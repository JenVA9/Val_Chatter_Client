use crate::app::App;

pub enum InputAction {
    Send { content: String, image_url: Option<String> },
    PickImage,
}

pub fn show(ui: &mut egui::Ui, app: &mut App) -> Option<InputAction> {
    let mut action = None;

    // Show pending image badge
    if let Some(url) = app.chat.pending_image_url.clone() {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(format!("[img] {}", url)).small());
            if ui.small_button("x").clicked() {
                app.chat.pending_image_url = None;
            }
        });
    }

    if app.pending_upload {
        ui.horizontal(|ui| {
            ui.spinner();
            ui.label(egui::RichText::new("Uploading…").small().weak());
        });
    }

    ui.horizontal(|ui| {
        let attach_btn = ui.button("[img]");
        if attach_btn.clicked() {
            action = Some(InputAction::PickImage);
        }

        let input_width = ui.available_width() - 70.0;
        let te = egui::TextEdit::singleline(&mut app.chat.input_buffer)
            .hint_text("Message…")
            .desired_width(input_width);
        let resp = ui.add(te);

        let send_ready = !app.chat.input_buffer.trim().is_empty()
            || app.chat.pending_image_url.is_some();

        let enter_pressed = resp.lost_focus()
            && ui.input(|i| i.key_pressed(egui::Key::Enter));

        let send_clicked = ui.add_enabled(send_ready, egui::Button::new("Send")).clicked();

        if (enter_pressed || send_clicked) && send_ready && action.is_none() {
            let content = std::mem::take(&mut app.chat.input_buffer);
            action = Some(InputAction::Send {
                content: content.trim().to_string(),
                image_url: None,
            });
        }
    });

    action
}
