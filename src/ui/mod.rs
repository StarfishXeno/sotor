use self::{editor_general::EditorGeneral, styles::set_button_styles, widgets::UiExt};
use crate::save::{Game, Save};
use egui::Ui;
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
    save: Save,
    tab: Tab,
}

impl Default for SotorApp {
    fn default() -> Self {
        let save = Save::read_from_directory("./save/", Game::One).unwrap();
        Self {
            save,
            tab: Tab::General,
        }
    }
}

impl SotorApp {
    pub fn new(ctx: &eframe::CreationContext<'_>) -> Self {
        styles::set_styles(&ctx.egui_ctx);
        SotorApp::default()
    }
}

impl eframe::App for SotorApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("save_tabs")
            .show_separator_line(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    set_button_styles(ui);
                    for tab in Tab::UNIT_VALUES {
                        let btn = ui.s_button(&tab.to_string(), self.tab == tab, false);
                        if btn.clicked() {
                            self.tab = tab;
                        }
                    }
                });
            });
        egui::CentralPanel::default().show(ctx, |ui| match self.tab {
            Tab::General => EditorGeneral::new(&mut self.save).show(ui),
            Tab::Globals => println!("globals"),
            _ => {}
        });
    }
}
