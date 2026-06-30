use crate::app::App;
use super::message;

pub fn show(_ctx: &egui::Context, ui: &mut egui::Ui, app: &mut App) {
    let scroll = egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .stick_to_bottom(app.chat.scroll_to_bottom);

    scroll.show(ui, |ui| {
        if app.chat.messages.is_empty() {
            ui.add_space(20.0);
            ui.centered_and_justified(|ui| {
                ui.label(egui::RichText::new("No messages yet. Be the first!").weak().italics());
            });
            return;
        }

        let my_user_id = app.auth.user_id;
        let server_url = app.server_url.clone();

        // Collect actions first to avoid borrow conflicts
        let mut to_delete: Option<i64> = None;
        let mut to_pin: Option<(i64, bool)> = None;
        let mut to_view: Option<String> = None;

        for msg in &app.chat.messages {
            let is_own = msg.user_id == my_user_id;
            let texture = msg.image_url.as_ref().and_then(|u| app.image_cache.get(u));

            let action = message::show(ui, msg, is_own, texture, &server_url);
            match action {
                Some(message::MsgAction::Delete(id)) => to_delete = Some(id),
                Some(message::MsgAction::Pin(id)) => to_pin = Some((id, true)),
                Some(message::MsgAction::Unpin(id)) => to_pin = Some((id, false)),
                Some(message::MsgAction::ViewImage(url)) => to_view = Some(url),
                None => {}
            }
        }

        if let Some(id) = to_delete { app.spawn_delete_message(id); }
        if let Some((id, pin)) = to_pin { app.spawn_pin_message(id, pin); }
        if let Some(url) = to_view { app.viewing_image = Some(url); }
    });

    app.chat.scroll_to_bottom = false;
}
