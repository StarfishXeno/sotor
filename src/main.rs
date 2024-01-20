mod save;
mod ui;
mod util;
include!(concat!(env!("OUT_DIR"), "/codegen.rs"));
fn main() -> eframe::Result<()> {
    let now = std::time::Instant::now();
    let a = default_game_data();
    println!("{:.2?}", now.elapsed());
    std::process::exit(0);
    #[cfg(target_endian = "big")]
    {
        compile_error!("nope");
    }
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
