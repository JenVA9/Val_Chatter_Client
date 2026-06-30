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

        let server_url = app.server_url.clone();

        let mut to_delete: Option<i64> = None;
        let mut to_pin_dialog: Option<i64> = None;
        let mut to_unpin: Option<i64> = None;
        let mut to_view: Option<String> = None;

        for msg in &app.chat.messages {
            let texture = msg.image_url.as_ref().and_then(|u| app.image_cache.get(u));
            let action = message::show(ui, msg, texture, &server_url, &mut app.scroll_to_message);
            match action {
                Some(message::MsgAction::Delete(id)) => to_delete = Some(id),
                Some(message::MsgAction::PinDialog(id)) => to_pin_dialog = Some(id),
                Some(message::MsgAction::Unpin(id)) => to_unpin = Some(id),
                Some(message::MsgAction::ViewImage(url)) => to_view = Some(url),
                None => {}
            }
        }

        if let Some(id) = to_delete { app.spawn_delete_message(id); }
        if let Some(id) = to_pin_dialog {
            app.pin_dialog = Some(crate::app::PinDialog {
                message_id: id,
                duration_str: String::new(),
                is_permanent: true,
            });
        }
        if let Some(id) = to_unpin { app.spawn_pin_message_timed(id, false, None); }
        if let Some(url) = to_view { app.viewing_image = Some(url); }
    });

    app.chat.scroll_to_bottom = false;
}
