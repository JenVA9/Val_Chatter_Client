use crate::app::App;

pub fn show(ctx: &egui::Context, app: &mut App) {
    let Some(url) = app.viewing_image.clone() else { return };

    let mut open = true;
    egui::Window::new("Image Viewer")
        .open(&mut open)
        .resizable(true)
        .default_size([700.0, 500.0])
        .collapsible(false)
        .show(ctx, |ui| {
            if let Some(texture) = app.image_cache.get(&url) {
                let max = ui.available_size();
                let size = texture.size_vec2();
                let scale = (max.x / size.x).min(max.y / size.y).min(1.0);
                let display = egui::Vec2::new(size.x * scale, size.y * scale);
                ui.add(egui::Image::new(texture).max_size(display));
            } else {
                ui.centered_and_justified(|ui| ui.spinner());
            }

            ui.separator();
            if ui.button("Close").clicked() {
                app.viewing_image = None;
            }
        });

    if !open {
        app.viewing_image = None;
    }
}
