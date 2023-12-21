use crate::{
    formats::{erf, gff},
    util::{read_file, get_erf_date}, ui::TemplateApp,
};

mod formats;
mod ui;
mod util;

fn main() -> eframe::Result<()> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([900.0, 540.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "Saves of The Old Republic",
        native_options,
        Box::new(|cc| Box::new(TemplateApp::new(cc))),
    )
}
