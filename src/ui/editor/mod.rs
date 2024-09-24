use crate::{
    save::Save,
    ui::{
        styles::set_button_styles,
        widgets::{Icon, UiExt as _},
        UiRef,
    },
    util::{ContextExt, Message},
};
use core::GameDataMapped;
use egui::Layout;
use emath::Align;
use macros::{EnumList, EnumToString};
use serde::{Deserialize, Serialize};

mod area;
mod characters;
mod general;
mod globals;
mod inventory;
mod quests;

pub fn editor_placeholder(ui: UiRef) {
    ui.horizontal_centered(|ui| {
        // TODO there should be a better way to center vertically
        ui.s_offset(ui.max_rect().width() / 2. - 100., 0.);
        let ctx = ui.ctx().clone();

        #[cfg(target_arch = "wasm32")]
        {
            use crate::util::{execute, select_save};
            use ahash::{HashMap, HashMapExt as _};
            let selecting = ctx.get_data_raw("ep_selecting").unwrap_or(false);
            if !selecting {
                execute(async move {
                    ctx.set_data_raw("ep_selecting", true);
                    if let Some(handles) = select_save().await {
                        let mut files = HashMap::with_capacity(handles.len());
                        for h in handles {
                            files.insert(h.file_name().to_lowercase(), h.read().await);
                        }

                        ctx.send_message(Message::LoadSaveFromFiles(files));
                        // something something ui thread sleeping without input
                        ctx.request_repaint();
                        ctx.set_data_raw("ep_selecting", false);
                    }
                });
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            ui.label("Please");

            set_button_styles(ui);
            let btn = ui.s_button_basic("Select");
            if btn.clicked() {
                if let Some(path) =
                    crate::util::select_directory("Select a save directory".to_owned())
                {
                    ctx.send_message(Message::LoadSaveFromDir(path));
                }
            }

            ui.label("a save");
        }
    });
}

#[derive(EnumToString, EnumList, Serialize, Deserialize, PartialEq, Clone, Copy, Default)]
#[repr(u8)]
pub enum Tab {
    #[default]
    General,
    Globals,
    Characters,
    Inventory,
    Quests,
    Area,
}

static TAB_ID: &str = "e_id";

pub struct Editor<'a> {
    save: &'a mut Save,
    data: &'a GameDataMapped,
}

impl<'a> Editor<'a> {
    pub fn new(save: &'a mut Save, data: &'a GameDataMapped) -> Self {
        Self { save, data }
    }

    pub fn show(&mut self, ui: UiRef) {
        let current_tab = ui.ctx().get_data_prs(TAB_ID).unwrap_or_default();

        ui.horizontal(|ui| {
            set_button_styles(ui);

            for tab in Tab::LIST {
                let btn = ui.s_button(&tab.to_string(), current_tab == tab, false);
                if btn.clicked() {
                    ui.ctx().set_data_prs(TAB_ID, tab);
                }
            }

            ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
                let btn = ui.s_icon_button(Icon::Leave, "Close save");
                if btn.clicked() {
                    ui.ctx().send_message(Message::CloseSave);
                }

                let btn = ui.s_icon_button(Icon::Save, "Save");
                if btn.clicked() {
                    ui.ctx().send_message(Message::Save);
                }

                let btn = ui.s_icon_button(Icon::Refresh, "Reload");
                if btn.clicked() {
                    ui.ctx().send_message(Message::ReloadSave);
                }
            })
        });

        ui.separator();

        match current_tab {
            Tab::General => general::Editor::new(self.save).show(ui),
            Tab::Globals => globals::Editor::new(self.save).show(ui),
            Tab::Characters => characters::Editor::new(self.save, self.data).show(ui),
            Tab::Quests => quests::Editor::new(self.save, self.data).show(ui),
            Tab::Inventory => inventory::Editor::new(self.save, self.data).show(ui),
            Tab::Area => area::Editor::new(self.save).show(ui),
        }
    }
}
