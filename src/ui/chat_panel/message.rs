use egui::TextureHandle;
use crate::models::Message;
use super::super::widgets::avatar;

pub enum MsgAction {
    Delete(i64),
    Pin(i64),
    Unpin(i64),
    ViewImage(String),
}

pub fn show(
    ui: &mut egui::Ui,
    msg: &Message,
    is_own: bool,
    texture: Option<&TextureHandle>,
    server_url: &str,
) -> Option<MsgAction> {
    let mut action = None;

    ui.add_space(4.0);

    let row_resp = ui.horizontal(|ui| {
        avatar::show(ui, &msg.username, 28.0);
        ui.vertical(|ui| {
            // Header: username + timestamp
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(&msg.username).strong().small());
                ui.label(egui::RichText::new(format_time(&msg.created_at)).weak().small());
                if msg.is_pinned {
                    ui.label(egui::RichText::new("📌").small());
                }
            });

            // Content
            if let Some(text) = &msg.content {
                if !text.is_empty() {
                    ui.label(text);
                }
            }

            // Image thumbnail
            if let Some(tex) = texture {
                let max_w = 300.0_f32;
                let size = tex.size_vec2();
                let scale = (max_w / size.x).min(1.0);
                let display_size = egui::Vec2::new(size.x * scale, size.y * scale);
                let img_resp = ui.add(
                    egui::Image::new(tex)
                        .max_size(display_size)
                        .sense(egui::Sense::click()),
                );
                if img_resp.clicked() {
                    let url = msg.image_url.as_deref().unwrap_or("");
                    let full = if url.starts_with("http") {
                        url.to_string()
                    } else {
                        format!("{}{}", server_url, url)
                    };
                    action = Some(MsgAction::ViewImage(full));
                }
            } else if msg.image_url.is_some() {
                ui.spinner();
            }
        });
    });

    // Context menu (right-click)
    let resp = row_resp.response;
    resp.context_menu(|ui| {
        if is_own {
            if ui.button("Delete").clicked() {
                action = Some(MsgAction::Delete(msg.id));
                ui.close_menu();
            }
        }
        if msg.is_pinned {
            if ui.button("Unpin").clicked() {
                action = Some(MsgAction::Unpin(msg.id));
                ui.close_menu();
            }
        } else {
            if ui.button("Pin").clicked() {
                action = Some(MsgAction::Pin(msg.id));
                ui.close_menu();
            }
        }
    });

    ui.separator();
    action
}

fn format_time(ts: &str) -> String {
    // ts is MySQL DATETIME like "2024-01-15T14:32:00.000Z"
    if ts.len() >= 16 {
        ts[11..16].to_string()
    } else {
        ts.to_string()
    }
}
