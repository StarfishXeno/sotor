use crate::{
    save::Game,
    ui::{
        styles::{BLUE, RED},
        widgets::Icon,
        widgets::UiExt,
        UiRef,
    },
    util::{ContextExt, Directory, Message},
};
use egui::{Layout, RichText};
use std::collections::HashMap;

pub struct SidePanel<'a> {
    game_data: &'a [Option<String>; Game::COUNT],
    save_list: &'a [HashMap<String, Vec<Directory>>; Game::COUNT],
}

impl<'a> SidePanel<'a> {
    pub fn new(
        game_data: &'a [Option<String>; Game::COUNT],
        save_list: &'a [HashMap<String, Vec<Directory>>; Game::COUNT],
    ) -> Self {
        Self {
            game_data,
            save_list,
        }
    }

    pub fn show(&self, ui: UiRef) {
        ui.s_offset([0., 3.]);
        ui.horizontal(|ui| self.header(ui));
        ui.separator();
    }

    fn header_game_label(&self, ui: UiRef, game: Game) {
        let (color, tooltip) = if self.game_data[0].is_some() {
            (BLUE, "Game data loaded")
        } else {
            (RED, "Game data missing, select valid game path in the settins")
        };
        ui.label(RichText::new(format!("K{}", game.to_idx() + 1)).color(color))
            .on_hover_text(tooltip);
    }

    fn header(&self, ui: UiRef) {
        Game::LIST.map(|game| self.header_game_label(ui, game));

        ui.with_layout(Layout::right_to_left(emath::Align::Center), |ui| {
            let settings_btn = ui.s_icon_button(Icon::Gear, "Settings");
            if settings_btn.clicked() {
                ui.ctx().send_message(Message::OpenSettings);
            }

            let reload_game_btn = ui.s_icon_button(Icon::Reload, "Reload game data");
            if reload_game_btn.clicked() {
                ui.ctx().send_message(Message::ReloadGameData);
            }

            let reload_saves_btn = ui.s_icon_button(Icon::Refresh, "Refresh save list");
            if reload_saves_btn.clicked() {
                ui.ctx().send_message(Message::ReloadSaveList);
            }
        });
    }
}
