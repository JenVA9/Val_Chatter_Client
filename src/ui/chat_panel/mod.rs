pub mod message_list;
pub mod message;
pub mod pinned_bar;
pub mod image_viewer;
pub mod chat_input;

use crate::app::App;

pub fn show(ctx: &egui::Context, ui: &mut egui::Ui, app: &mut App) {
    if app.chat.thread_id.is_none() && app.nav.is_empty() {
        ui.centered_and_justified(|ui| {
            ui.label(egui::RichText::new("Select nodes from the navigation panel\nto open a thread").weak().italics());
        });
        return;
    }

    if app.chat.thread_id.is_none() {
        ui.centered_and_justified(|ui| {
            ui.spinner();
        });
        return;
    }

    // Pinned messages bar at top
    if !app.chat.pinned.is_empty() {
        egui::TopBottomPanel::top("pinned_bar")
            .frame(egui::Frame::default().inner_margin(6.0).fill(ui.visuals().faint_bg_color))
            .show_inside(ui, |ui| {
                pinned_bar::show(ui, &app.chat.pinned);
            });
    }

    // Chat input at bottom
    egui::TopBottomPanel::bottom("chat_input")
        .frame(egui::Frame::default().inner_margin(8.0))
        .show_inside(ui, |ui| {
            if let Some(action) = chat_input::show(ui, app) {
                match action {
                    chat_input::InputAction::Send { content, image_url } => {
                        let img = app.chat.pending_image_url.take().or(image_url);
                        app.spawn_send_message(content, img);
                        app.chat.scroll_to_bottom = true;
                    }
                    chat_input::InputAction::PickImage => {
                        let path = rfd::FileDialog::new()
                            .add_filter("Images", &["png", "jpg", "jpeg", "gif", "webp"])
                            .pick_file();
                        if let Some(p) = path {
                            app.spawn_upload_image(p);
                        }
                    }
                }
            }
        });

    // Message list fills remaining space
    egui::CentralPanel::default()
        .frame(egui::Frame::default().inner_margin(0.0))
        .show_inside(ui, |ui| {
            message_list::show(ctx, ui, app);
        });
}
