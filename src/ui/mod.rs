use self::{editor_general::EditorGeneral, styles::set_button_styles, widgets::UiExt};
use crate::save::{Game, Save};
use egui::{panel::Side, Layout, TextureHandle, TextureOptions, Ui};
use emath::Align;
use image::io::Reader as ImageReader;
use sotor_macros::EnumToString;

mod editor_general;
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
    save: Save,
    tab: Tab,
}

impl SotorApp {
    pub fn new(ctx: &eframe::CreationContext<'_>) -> Self {
        let save = Save::read_from_directory("./save/", Game::One).unwrap();
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
            ui.horizontal(|ui| {
                set_button_styles(ui);
                for tab in Tab::UNIT_VALUES {
                    let btn = ui.s_button(&tab.to_string(), self.tab == tab, false);
                    if btn.clicked() {
                        self.tab = tab;
                    }
                }
                ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
                    set_button_styles(ui);
                    let btn = ui.s_button_basic("Save");
                    if btn.clicked() {
                        self.save = Save::save_to_directory("./save", self.save.clone()).unwrap();
                    }
                })
            });
            ui.separator();
            match self.tab {
                Tab::General => EditorGeneral::new(&mut self.save, &self.texture).show(ui),
                Tab::Globals => println!("globals"),
                _ => {}
            }
        });
    }
}
