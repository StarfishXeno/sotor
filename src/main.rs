use crate::ui::SotorApp;

mod formats;
mod save;
mod ui;
mod util;

fn main() -> eframe::Result<()> {
    env_logger::init();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_min_inner_size([900., 540.]),
        ..Default::default()
    };

    eframe::run_native(
        "Saves of The Old Republic",
        native_options,
        Box::new(|cc| Box::new(SotorApp::new(cc))),
    )
}
