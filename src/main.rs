mod formats;
mod save;
mod ui;
mod util;

fn main() -> eframe::Result<()> {
    env_logger::init();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_app_id("sotor")
            .with_min_inner_size([900., 540.])
            .with_icon(util::load_icon()),
        ..Default::default()
    };

    eframe::run_native(
        "Saves of the Old Republic",
        native_options,
        Box::new(|cc| Box::new(ui::SotorApp::new(cc))),
    )
}
