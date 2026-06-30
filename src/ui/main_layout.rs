use crate::app::App;

pub fn show(ctx: &egui::Context, app: &mut App) {
    // Top bar: username + logout
    egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("VAL TACTICS").strong());
            ui.separator();
            ui.label(&app.auth.username);
            if ui.small_button("Logout").clicked() {
                app.logout();
                return;
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(egui::RichText::new(&app.server_url).weak().small());
            });
        });
    });

    // Nav panel on the right
    let nav_changed = egui::SidePanel::right("nav_panel")
        .min_width(260.0)
        .max_width(360.0)
        .show(ctx, |ui| {
            crate::ui::nav_panel::show(ui, &app.nodes, &mut app.nav)
        })
        .inner;

    // Image viewer overlay (modal window)
    if app.viewing_image.is_some() {
        crate::ui::chat_panel::image_viewer::show(ctx, app);
    }

    // Chat panel fills the rest
    egui::CentralPanel::default().show(ctx, |ui| {
        crate::ui::chat_panel::show(ctx, ui, app);
    });

    let _ = nav_changed;
}
