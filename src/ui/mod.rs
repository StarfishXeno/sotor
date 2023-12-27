use std::process;

use crate::save::Save;
use egui::{panel::Side, Ui};
use sotor_macros::EnumToString;

mod editor;
mod save_select;
mod styles;
mod widgets;

type UiRef<'a> = &'a mut Ui;

#[derive(EnumToString, PartialEq, Clone, Copy)]
enum Tab {
    General,
    Globals,
    Characters,
    Inventory,
    Quests,
}
pub struct SotorApp {
    save: Option<Save>,
    tab: Tab,
}

impl SotorApp {
    pub fn new(ctx: &eframe::CreationContext<'_>) -> Self {
        styles::set_styles(&ctx.egui_ctx);
        let path = "./assets/k2/saves/000000 - QUICKSAVE/";
        match Save::read_from_directory(path, &ctx.egui_ctx) {
            Ok(save) => Self {
                save: Some(save),
                tab: Tab::General,
            },
            Err(err) => {
                println!("{err}");
                process::exit(1);
            }
        }
    }
}

impl eframe::App for SotorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::new(Side::Left, "save_select").show(ctx, |_ui| {});

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(save) = &mut self.save {
                editor::Editor::new(save, &mut self.tab).show(ui);
            } else {
                editor::EditorPlaceholder::new().show(ui);
            };
        });
    }
}
