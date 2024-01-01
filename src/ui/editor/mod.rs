use crate::{
    save::Save,
    util::{select_directory, ContextExt, Message},
};
use egui::Layout;
use emath::Align;
use sotor_macros::{EnumList, EnumToString};

use super::{
    styles::set_button_styles,
    widgets::{Icon, UiExt as _},
    UiRef,
};

mod general;
mod globals;
mod quests;

pub fn editor_placeholder(ui: UiRef) {
    ui.horizontal_centered(|ui| {
        // TODO there should be a better way to center vertically
        ui.s_offset([ui.max_rect().width() / 2. - 100., 0.]);

        ui.label("Please");

        set_button_styles(ui);
        let btn = ui.s_button_basic("Select");
        if btn.clicked() {
            let dir = select_directory("Select a save directory".to_owned());
            if let Some(handle) = dir {
                let path = handle.path().to_str().unwrap().to_owned();

                ui.ctx().send_message(Message::LoadSaveFromDir(path));
            }
        }

        ui.label("a save");
    });
}

#[derive(EnumToString, EnumList, PartialEq, Clone, Copy, Default)]
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
            set_button_styles(ui);

            for tab in Tab::LIST {
                let btn = ui.s_button(&tab.to_string(), current_tab == tab, false);
                if btn.clicked() {
                    ui.ctx().set_data(TAB_ID, tab);
                }
            }

            ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
                let btn = ui.s_icon_button(Icon::Leave, "Close save");
                if btn.clicked() {
                    ui.ctx().send_message(Message::CloseSave);
                }

                let btn = ui.s_icon_button(Icon::Save, "Save");
                if btn.clicked() {
                    Save::save_to_directory("./save", self.save).unwrap();
                }

                let btn = ui.s_icon_button(Icon::Refresh, "Reload");
                if btn.clicked() {
                    ui.ctx().send_message(Message::ReloadSave);
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
