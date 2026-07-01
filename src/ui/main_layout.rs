use crate::app::App;

pub fn show(ctx: &egui::Context, app: &mut App) {
    // Top bar
    egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("VAL TACTICS").strong());
            ui.separator();
            ui.label(&app.auth.username);
            if app.is_admin {
                ui.label(egui::RichText::new("★").color(egui::Color32::from_rgb(255, 200, 50)));
            }
            if app.is_guest {
                ui.label(egui::RichText::new("(guest)").weak().small());
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(egui::RichText::new(&app.server_url).weak().small());
                ui.separator();
                if ui.small_button("⚙").on_hover_text("Settings").clicked() {
                    app.settings_open = true;
                }
                if app.is_admin {
                    if ui.small_button("🛡").on_hover_text("Admin Panel").clicked() {
                        app.admin_panel_open = true;
                        app.spawn_load_admin_data();
                    }
                }
                if app.input_locked {
                    ui.label(egui::RichText::new("⚠ STORAGE FULL").color(egui::Color32::from_rgb(255, 100, 0)).small());
                }
            });
        });
    });

    // Nav panel on the right
    let nav_changed = egui::SidePanel::right("nav_panel")
        .min_width(260.0)
        .max_width(360.0)
        .show(ctx, |ui| {
            crate::ui::nav_panel::show(ui, &app.nodes, &mut app.nav, &app.active_nodes, &mut app.collapsed_sections)
        })
        .inner;

    let _ = nav_changed;

    // Image viewer overlay
    if app.viewing_image.is_some() {
        crate::ui::chat_panel::image_viewer::show(ctx, app);
    }

    // Settings and admin panel overlays
    crate::ui::settings::show(ctx, app);
    crate::ui::admin_panel::show(ctx, app);

    // Chat panel fills the rest
    egui::CentralPanel::default().show(ctx, |ui| {
        crate::ui::chat_panel::show(ctx, ui, app);
    });
}
