use egui::{Layout, TextureHandle};
use emath::Align;

use crate::save::Save;

use super::{styles::set_button_styles, widgets::UiExt as _, Tab, UiRef};

mod general;
mod globals;
mod quests;

pub struct Editor<'a> {
    save: &'a mut Save,
    tab: &'a mut Tab,
    texture: &'a TextureHandle,
}

impl<'a> Editor<'a> {
    pub fn new(save: &'a mut Save, tab: &'a mut Tab, texture: &'a TextureHandle) -> Self {
        Self { save, tab, texture }
    }
    pub fn show(&mut self, ui: UiRef) {
        ui.horizontal(|ui| {
            set_button_styles(ui);
            for tab in Tab::UNIT_VALUES {
                let btn = ui.s_button(&tab.to_string(), *self.tab == tab, false);
                if btn.clicked() {
                    *self.tab = tab;
                }
            }
            ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
                set_button_styles(ui);
                let btn = ui.s_button_basic("Save");
                if btn.clicked() {
                    *self.save = Save::save_to_directory("./save", self.save.clone()).unwrap();
                }
            })
        });
        ui.separator();
        match self.tab {
            Tab::General => general::EditorGeneral::new(self.save, self.texture).show(ui),
            Tab::Globals => globals::EditorGlobals::new(&mut self.save.globals).show(ui),
            Tab::Quests => quests::EditorQuests::new(&mut self.save.party_table.journal).show(ui),

            _ => {}
        }
    }
}
