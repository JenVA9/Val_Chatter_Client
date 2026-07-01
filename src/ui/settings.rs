use crate::app::{App, Theme};

pub fn show(ctx: &egui::Context, app: &mut App) {
    if !app.settings_open { return; }

    let mut open = app.settings_open;

    egui::Window::new("⚙  Settings")
        .open(&mut open)
        .resizable(true)
        .min_width(360.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                // ── Account ───────────────────────────────────────────────
                ui.collapsing("Account", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Signed in as:");
                        ui.label(egui::RichText::new(&app.auth.username).strong());
                        if app.is_admin {
                            ui.label(egui::RichText::new("[Admin]").color(egui::Color32::from_rgb(255, 180, 0)).small());
                        }
                    });
                    if ui.button("Log out").clicked() {
                        app.logout();
                        app.settings_open = false;
                    }
                });

                ui.separator();

                // ── Servers ───────────────────────────────────────────────
                ui.collapsing("Server Connections", |ui| {
                    let servers_copy = app.servers.clone();
                    for (i, srv) in servers_copy.iter().enumerate() {
                        ui.horizontal(|ui| {
                            let is_active = srv.url == app.server_url;
                            if is_active {
                                ui.label(egui::RichText::new("●").color(egui::Color32::from_rgb(80, 200, 80)));
                            } else {
                                ui.label(egui::RichText::new("○").weak());
                            }
                            ui.label(&srv.url);
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.small_button("✕").clicked() {
                                    app.servers.remove(i);
                                }
                                if !is_active && ui.small_button("Connect").clicked() {
                                    app.switch_server(i);
                                }
                            });
                        });
                    }
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut app.new_server_url_input)
                                .hint_text("http://server:3000")
                                .desired_width(220.0),
                        );
                        if ui.small_button("Add").clicked() && !app.new_server_url_input.trim().is_empty() {
                            let url = std::mem::take(&mut app.new_server_url_input).trim().to_string();
                            if !app.servers.iter().any(|s| s.url == url) {
                                app.servers.push(crate::app::ServerProfile {
                                    url,
                                    ..Default::default()
                                });
                            }
                        }
                    });
                });

                ui.separator();

                // ── Appearance ────────────────────────────────────────────
                ui.collapsing("Appearance", |ui| {
                    ui.label("Theme:");
                    ui.horizontal_wrapped(|ui| {
                        for (label, theme) in &[
                            ("Dark", Theme::Dark),
                            ("Light", Theme::Light),
                            ("Nebula", Theme::Nebula),
                            ("OLED", Theme::OledBlack),
                        ] {
                            let selected = &app.theme == theme;
                            let btn = egui::Button::new(*label)
                                .fill(if selected { egui::Color32::from_rgb(40, 90, 140) } else { egui::Color32::TRANSPARENT });
                            if ui.add(btn).clicked() {
                                app.theme = theme.clone();
                                app.apply_theme(ctx);
                            }
                        }
                    });

                    ui.add_space(8.0);
                    ui.label("Font size:");
                    if ui.add(egui::Slider::new(&mut app.font_size, 10.0..=24.0).suffix("px")).changed() {
                        app.apply_theme(ctx);
                    }
                });

                ui.separator();

                // ── Admin ─────────────────────────────────────────────────
                if app.is_admin {
                    if ui.button("🛡  Open Admin Panel").clicked() {
                        app.admin_panel_open = true;
                        app.settings_open = false;
                        app.spawn_load_admin_data();
                    }
                }
            });
        });

    app.settings_open = open;
}
