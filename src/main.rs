use egui::IconData;

mod formats;
mod save;
mod ui;
mod util;

fn main() -> eframe::Result<()> {
    env_logger::init();

    let rgba = image::load_from_memory(include_bytes!("../assets/hk.png"))
        .unwrap()
        .to_rgba8();
    let (width, height) = rgba.dimensions();
    let icon = IconData {
        rgba: rgba.to_vec(),
        width,
        height,
    };

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_app_id("sotor")
            .with_min_inner_size([900., 540.])
            .with_icon(icon),
        ..Default::default()
    };

    eframe::run_native(
        "Saves of the Old Republic",
        native_options,
        Box::new(|cc| Box::new(ui::SotorApp::new(cc))),
    )
}
