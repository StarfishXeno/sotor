use std::collections::HashMap;

use formats::{
    erf::{self, Erf},
    gff::{self, Gff, Struct},
};

use crate::ui::SotorApp;

mod formats;
mod save;
mod ui;
mod util;

fn main() -> eframe::Result<()> {
    env_logger::init();
    // just shutup about unused writes for now
    erf::write(Erf {
        file_type: "a".into(),
        file_version: "a".into(),
        build_day: 0,
        build_year: 0,
        resources: HashMap::new(),
        loc_strings: vec![],
        description_str_ref: 0,
    });
    gff::write(Gff {
        file_type: "a".into(),
        file_version: "a".into(),
        content: Struct {
            tp: 0,
            fields: HashMap::new(),
        },
    });

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([900.0, 540.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Saves of The Old Republic",
        native_options,
        Box::new(|cc| Box::new(SotorApp::new(cc))),
    )
}
