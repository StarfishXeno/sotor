use crate::save::{Game, Save};
use egui::{panel::Side, TextureHandle, TextureOptions, Ui};
use image::io::Reader as ImageReader;
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
    texture: TextureHandle,
    save: Option<Save>,
    tab: Tab,
}

impl SotorApp {
    pub fn new(ctx: &eframe::CreationContext<'_>) -> Self {
        let save = Save::read_from_directory("./save/", Game::One).ok();
        styles::set_styles(&ctx.egui_ctx);
        let img = ImageReader::open("./save/screen.tga")
            .unwrap()
            .decode()
            .unwrap();
        let size = [img.width() as _, img.height() as _];
        let data = img.to_rgba8();
        let data = data.as_flat_samples();
        let image = egui::ColorImage::from_rgba_unmultiplied(size, data.as_slice());

        let texture = ctx
            .egui_ctx
            .load_texture("save_image", image, TextureOptions::NEAREST);

        Self {
            texture,
            save,
            tab: Tab::General,
        }
    }
}

impl eframe::App for SotorApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::new(Side::Left, "save_select").show(ctx, |_ui| {});
        egui::CentralPanel::default().show(ctx, |ui| {
            let Some(save) = &mut self.save else {
                return;
            };
            editor::Editor::new(save, &mut self.tab, &self.texture).show(ui);
        });
    }
}
