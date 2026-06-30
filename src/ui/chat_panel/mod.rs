pub mod message_list;
pub mod message;
pub mod pinned_bar;
pub mod image_viewer;
pub mod chat_input;
pub mod whiteboard;

use crate::app::{App, ThreadMode};

pub fn show(ctx: &egui::Context, ui: &mut egui::Ui, app: &mut App) {
    if app.chat.thread_id.is_none() && app.nav.is_empty() {
        ui.centered_and_justified(|ui| {
            ui.label(egui::RichText::new("Select nodes from the navigation panel\nto open a thread").weak().italics());
        });
        return;
    }

    if app.chat.thread_id.is_none() {
        ui.centered_and_justified(|ui| { ui.spinner(); });
        return;
    }

    let thread_id = app.chat.thread_id.unwrap();
    let mode = app.thread_mode.get(&thread_id).cloned().unwrap_or(ThreadMode::Chat);

    // ── Mode toggle bar ───────────────────────────────────────────────────
    if !app.is_guest {
        egui::TopBottomPanel::top("mode_toggle")
            .frame(egui::Frame::default().inner_margin(egui::Margin::symmetric(8.0, 4.0)))
            .show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    let chat_sel = mode == ThreadMode::Chat;
                    let wb_sel = mode == ThreadMode::Whiteboard;

                    let chat_btn = egui::Button::new("💬 Chat")
                        .fill(if chat_sel { egui::Color32::from_rgb(40, 90, 140) } else { egui::Color32::TRANSPARENT });
                    if ui.add(chat_btn).clicked() && !chat_sel {
                        app.thread_mode.insert(thread_id, ThreadMode::Chat);
                    }

                    let wb_btn = egui::Button::new("🎨 Whiteboard")
                        .fill(if wb_sel { egui::Color32::from_rgb(40, 90, 140) } else { egui::Color32::TRANSPARENT });
                    if ui.add(wb_btn).clicked() && !wb_sel {
                        app.thread_mode.insert(thread_id, ThreadMode::Whiteboard);
                        app.spawn_load_whiteboard(thread_id);
                    }
                });
            });
    }

    // ── Whiteboard mode ───────────────────────────────────────────────────
    if mode == ThreadMode::Whiteboard {
        egui::CentralPanel::default()
            .frame(egui::Frame::default().inner_margin(0.0))
            .show_inside(ui, |ui| {
                whiteboard::show(ctx, ui, app);
            });
        return;
    }

    // ── Chat mode ─────────────────────────────────────────────────────────

    // Pinned messages bar at top
    if !app.chat.pinned.is_empty() {
        egui::TopBottomPanel::top("pinned_bar")
            .frame(egui::Frame::default().inner_margin(6.0).fill(ui.visuals().faint_bg_color))
            .show_inside(ui, |ui| {
                let pb_action = pinned_bar::show(ui, &app.chat.pinned);
                if let Some(id) = pb_action.jump_to {
                    app.scroll_to_message = Some(id);
                }
            });
    }

    // Chat input at bottom (or locked notice)
    egui::TopBottomPanel::bottom("chat_input")
        .frame(egui::Frame::default().inner_margin(8.0))
        .show_inside(ui, |ui| {
            if app.is_guest {
                ui.centered_and_justified(|ui| {
                    ui.label(egui::RichText::new("👁  Guest mode — view only").weak().italics());
                });
            } else if app.input_locked {
                ui.centered_and_justified(|ui| {
                    ui.label(
                        egui::RichText::new("⚠  Server storage full — messaging disabled")
                            .color(egui::Color32::from_rgb(255, 180, 0)),
                    );
                });
            } else if let Some(action) = chat_input::show(ui, app) {
                match action {
                    chat_input::InputAction::Send { content, .. } => {
                        let img = app.chat.pending_image_url.take();
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

    // Message list
    egui::CentralPanel::default()
        .frame(egui::Frame::default().inner_margin(0.0))
        .show_inside(ui, |ui| {
            message_list::show(ctx, ui, app);
        });

    // ── Pin dialog ────────────────────────────────────────────────────────
    show_pin_dialog(ctx, app);
}

fn show_pin_dialog(ctx: &egui::Context, app: &mut App) {
    if app.pin_dialog.is_none() { return; }

    let mut open = true;
    let mut do_pin = false;
    let mut do_cancel = false;

    egui::Window::new("Pin Message")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .open(&mut open)
        .show(ctx, |ui| {
            let dialog = app.pin_dialog.as_mut().unwrap();

            ui.checkbox(&mut dialog.is_permanent, "Pin forever");

            if !dialog.is_permanent {
                ui.horizontal(|ui| {
                    ui.label("Duration:");
                    ui.add(
                        egui::TextEdit::singleline(&mut dialog.duration_str)
                            .hint_text("e.g. 30m, 2h, 1d")
                            .desired_width(100.0),
                    );
                });
                ui.label(egui::RichText::new("Formats: 30m, 2h, 12h, 1d").small().weak());
            }

            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Pin").clicked() { do_pin = true; }
                if ui.button("Cancel").clicked() { do_cancel = true; }
            });
        });

    if do_pin {
        if let Some(dialog) = app.pin_dialog.take() {
            let minutes = if dialog.is_permanent { None } else { parse_duration(&dialog.duration_str) };
            app.spawn_pin_message_timed(dialog.message_id, true, minutes);
        }
    } else if !open || do_cancel {
        app.pin_dialog = None;
    }
}

fn parse_duration(s: &str) -> Option<u32> {
    let s = s.trim().to_lowercase();
    if s.ends_with('d') {
        s[..s.len()-1].parse::<u32>().ok().map(|d| d * 24 * 60)
    } else if s.ends_with('h') {
        s[..s.len()-1].parse::<u32>().ok().map(|h| h * 60)
    } else if s.ends_with('m') {
        s[..s.len()-1].parse::<u32>().ok()
    } else {
        s.parse::<u32>().ok()
    }
}
