use egui::{Color32, Rect, Response, TextBuffer, Ui, Vec2};
use egui_extras::{Size, StripBuilder};
use sotor_macros::EnumToString;

use crate::util::{seconds_to_time, Save};

#[derive(EnumToString, PartialEq, Clone, Copy)]
enum Tab {
    General,
    Globals,
    Characters,
    Inventory,
    Quests,
}

pub struct TemplateApp {
    save: Save,
    tab: Tab,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            save: Save::read_from_directory("./save/").unwrap(),
            tab: Tab::General,
        }
    }
}

fn text_edit(ui: &mut Ui, text: &mut dyn TextBuffer, enabled: bool) -> Response {
    ui.add_enabled(
        enabled,
        egui::TextEdit::singleline(text)
            .vertical_align(egui::Align::Center)
            .text_color(Color32::WHITE)
            .min_size(Vec2::new(300.0, 0.0)),
    )
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(_: &eframe::CreationContext<'_>) -> Self {
        TemplateApp::default()
    }

    fn show_general(&mut self, ui: &mut Ui) {
        let nfo = &mut self.save.nfo;
        egui::Grid::new("save_general")
            .num_columns(4)
            .spacing([20.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                ui.label("Save name: ");
                text_edit(ui, &mut nfo.save_name, true);
                ui.label("Area name: ");
                text_edit(ui, &mut nfo.area_name, false);
                ui.end_row();

                ui.label("Time played: ");
                text_edit(ui, &mut seconds_to_time(nfo.time_played), false);
                ui.label("Cheats used: ");
                ui.checkbox(&mut nfo.cheats_used, "");
                ui.end_row();

                ui.label("Credits: ");
                ui.label("Party XP: ");
                ui.checkbox(&mut nfo.cheats_used, "");
                ui.end_row();
            });
    }
}

impl eframe::App for TemplateApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("save_tabs").show(ctx, |ui| {
            ui.horizontal(|ui| {
                for val in Tab::UNIT_VALUES {
                    let btn = ui.button(val.to_string());
                    if btn.clicked() {
                        self.tab = val;
                    }
                    if self.tab == val {
                        btn.highlight();
                    }
                }
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| match self.tab {
            Tab::General => self.show_general(ui),
            Tab::Globals => {}
            _ => {}
        });
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}
