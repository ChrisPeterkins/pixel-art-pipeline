mod app;
mod drag_drop;
mod panels;
mod state;
mod texture;

use app::PixelpipeApp;

fn main() -> eframe::Result<()> {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_drag_and_drop(true),
        ..Default::default()
    };

    eframe::run_native(
        "pixelpipe",
        options,
        Box::new(|cc| Ok(Box::new(PixelpipeApp::new(cc)))),
    )
}
