use crate::{
    save::Game,
    ui::{
        styles::{BLUE, RED, WHITE},
        widgets::{color_text, Icon, UiExt},
        SaveDirectories, UiRef,
    },
    util::{ContextExt, Message},
};
use egui::{CursorIcon, Layout, ScrollArea};

pub struct SidePanel<'a> {
    game_data: &'a [Option<String>; Game::COUNT],
    save_list: &'a [Vec<SaveDirectories>; Game::COUNT],
}

impl<'a> SidePanel<'a> {
    pub fn new(
        game_data: &'a [Option<String>; Game::COUNT],
        save_list: &'a [Vec<SaveDirectories>; Game::COUNT],
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
        self.lists(ui);
    }

    fn header_game_label(&self, ui: UiRef, game: Game) {
        let (color, tooltip) = if self.game_data[game.to_idx()].is_some() {
            (BLUE, "Game data loaded")
        } else {
            (
                RED,
                "Game data missing, select a valid game path in the settings",
            )
        };
        ui.label(color_text(&format!("K{}", game.to_idx() + 1), color))
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

    fn lists(&self, ui: UiRef) {
        ui.set_width(ui.available_width());
        ScrollArea::vertical()
            .id_source("side_panel_scroll")
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                Game::LIST.map(|game| self.list(ui, game));
            });
    }

    fn list(&self, ui: UiRef, game: Game) {
        let game = game.to_idx();
        ui.label(format!("KOTOR {}", game + 1));

        for group in &self.save_list[game] {
            for save in &group.dirs {
                let mut selected = false;
                ui.toggle_value(&mut selected, color_text(&save.name, WHITE))
                    .on_hover_cursor(CursorIcon::PointingHand);
                if selected {
                    ui.ctx()
                        .send_message(Message::LoadSaveFromDir(save.path.clone()));
                }
            }
        }
    }
}
