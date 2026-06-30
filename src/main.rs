mod models;
mod state;
mod api;
mod ws;
mod utils;
mod ui;
mod app;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([900.0, 600.0])
            .with_title("Val Tactics"),
        ..Default::default()
    };
    eframe::run_native(
        "Val Tactics",
        options,
        Box::new(|cc| Box::new(app::App::new(cc))),
    )
}
