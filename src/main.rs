#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod save;
mod ui;
mod util;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    simplelog::TermLogger::init(
        if cfg!(debug_assertions) {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Info
        },
        simplelog::Config::default(),
        simplelog::TerminalMode::Stderr,
        simplelog::ColorChoice::Auto,
    )
    .unwrap();

    #[cfg(target_endian = "big")]
    {
        compile_error!("nope");
    }
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_app_id("sotor")
            .with_min_inner_size([960., 540.])
            .with_icon(util::load_icon()),
        ..Default::default()
    };

    eframe::run_native(
        "Saves of the Old Republic",
        native_options,
        Box::new(|cc| Box::new(ui::SotorApp::new(cc))),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "canvas", // hardcode it
                web_options,
                Box::new(|cc| Box::new(ui::SotorApp::new(cc))),
            )
            .await
            .expect("failed to start eframe");
    });
}
