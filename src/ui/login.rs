use egui::{Align, CentralPanel, Layout, RichText, Vec2};
use crate::app::App;

pub fn show(ctx: &egui::Context, app: &mut App) {
    CentralPanel::default().show(ctx, |ui| {
        ui.with_layout(Layout::top_down(Align::Center), |ui| {
            ui.add_space(80.0);

            ui.label(RichText::new("VAL TACTICS").size(36.0).strong());
            ui.add_space(8.0);
            ui.label(RichText::new("Team strategy chat").size(14.0).weak());
            ui.add_space(40.0);

            let form_width = 340.0;
            ui.allocate_ui(Vec2::new(form_width, 0.0), |ui| {
                ui.label("Server URL");
                let url_resp = ui.text_edit_singleline(&mut app.login_server_url);
                if url_resp.lost_focus() && app.login_server_url != app.server_url {
                    let url = app.login_server_url.trim().to_string();
                    app.update_server_url(url);
                }
                ui.add_space(12.0);

                ui.label("Username");
                ui.text_edit_singleline(&mut app.login_username);
                ui.add_space(8.0);

                ui.label("Password");
                ui.add(egui::TextEdit::singleline(&mut app.login_password).password(true));
                ui.add_space(16.0);

                let button_label = if app.login_register { "Register" } else { "Login" };
                let submit = ui.add_sized(
                    Vec2::new(form_width, 36.0),
                    egui::Button::new(button_label),
                );

                // Also trigger on Enter key
                let enter = ui.input(|i| i.key_pressed(egui::Key::Enter));
                if submit.clicked() || enter {
                    app.login_error = None;
                    let username = app.login_username.trim().to_string();
                    let password = app.login_password.clone();
                    let register = app.login_register;
                    app.spawn_login(username, password, register);
                }

                ui.add_space(8.0);
                let toggle_label = if app.login_register {
                    "Already have an account? Log in"
                } else {
                    "No account? Register"
                };
                if ui.small_button(toggle_label).clicked() {
                    app.login_register = !app.login_register;
                    app.login_error = None;
                }

                if let Some(err) = &app.login_error {
                    ui.add_space(12.0);
                    ui.colored_label(egui::Color32::RED, err);
                }
            });
        });
    });
}
