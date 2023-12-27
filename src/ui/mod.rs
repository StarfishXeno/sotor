use crate::{
    save::{Game, Save},
    util::load_tga,
};
use egui::{panel::Side, TextureHandle, TextureOptions, Ui};
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
    texture: Option<TextureHandle>,
    save: Option<Save>,
    tab: Tab,
}

impl SotorApp {
    pub fn new(ctx: &eframe::CreationContext<'_>) -> Self {
        let path = "./save/";
        let save = Save::read_from_directory(path, Game::One).ok();
        styles::set_styles(&ctx.egui_ctx);

        let tga = load_tga(path);
        let texture = tga.map(|i| {
            ctx.egui_ctx
                .load_texture("save_image", i, TextureOptions::NEAREST)
        });

        Self {
            texture,
            save,
            tab: Tab::General,
        }
    }
}

impl eframe::App for SotorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::new(Side::Left, "save_select").show(ctx, |_ui| {});

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(save) = &mut self.save {
                editor::Editor::new(save, &mut self.tab, &self.texture).show(ui);
            } else {
                editor::EditorPlaceholder::new().show(ui);
            };
        });
    }
}
