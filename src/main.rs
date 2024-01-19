mod formats;
mod game_data;
mod save;
mod ui;
mod util;

fn main() -> eframe::Result<()> {
    game_data::GameData::read(
        util::Game::One,
        "/mnt/media/SteamLibrary/steamapps/common/swkotor",
        None,
    )
    .unwrap();
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
