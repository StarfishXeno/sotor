use crate::{
    save::Save,
    util::{ContextExt, Message},
};
use egui::{Direction, Layout};
use emath::Align;
use sotor_macros::EnumToString;

use super::{styles::set_button_styles, widgets::UiExt as _, UiRef};

mod general;
mod globals;
mod quests;

pub fn editor_placeholder(ui: UiRef) {
    ui.with_layout(Layout::centered_and_justified(Direction::TopDown), |ui| {
        ui.label("Please select a save")
    });
}

#[derive(EnumToString, PartialEq, Clone, Copy, Default)]
pub enum Tab {
    #[default]
    General,
    Globals,
    Characters,
    Inventory,
    Quests,
}

static TAB_ID: &str = "editor_tab_id";

pub struct Editor<'a> {
    save: &'a mut Save,
}

impl<'a> Editor<'a> {
    pub fn new(save: &'a mut Save) -> Self {
        Self { save }
    }

    pub fn show(&mut self, ui: UiRef) {
        let current_tab = ui.ctx().get_data(TAB_ID).unwrap_or_default();

        ui.horizontal(|ui| {
            let channel = ui.ctx().get_channel();
            set_button_styles(ui);

            for tab in Tab::UNIT_VALUES {
                let btn = ui.s_button(&tab.to_string(), current_tab == tab, false);
                if btn.clicked() {
                    ui.ctx().set_data(TAB_ID, tab);
                }
            }

            ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
                let btn = ui.s_button_basic("Save");
                if btn.clicked() {
                    Save::save_to_directory("./save", self.save).unwrap();
                }

                let btn = ui.s_button_basic("Reload");
                if btn.clicked() {
                    channel.send(Message::ReloadSave).unwrap();
                }
            })
        });

        ui.separator();

        match current_tab {
            Tab::General => general::EditorGeneral::new(self.save).show(ui),
            Tab::Globals => globals::EditorGlobals::new(&mut self.save.globals).show(ui),
            Tab::Quests => quests::EditorQuests::new(&mut self.save.party_table.journal).show(ui),

            _ => {}
        }
    }
}
