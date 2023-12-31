use crate::{
    save::Game,
    ui::{
        styles::{BLUE, RED, WHITE},
        widgets::{color_text, Icon, UiExt},
        SaveDirectories, UiRef,
    },
    util::{ContextExt, Message},
};
use egui::{collapsing_header::CollapsingState, Frame, Layout, Margin, ScrollArea};

use super::styles::{GREEN_DARK, GREY};

pub struct SidePanel<'a> {
    current_save: &'a Option<String>,
    game_data: &'a [Option<String>; Game::COUNT],
    save_list: &'a [Vec<SaveDirectories>; Game::COUNT],
}

impl<'a> SidePanel<'a> {
    pub fn new(
        current_save: &'a Option<String>,
        game_data: &'a [Option<String>; Game::COUNT],
        save_list: &'a [Vec<SaveDirectories>; Game::COUNT],
    ) -> Self {
        Self {
            current_save,
            game_data,
            save_list,
        }
    }

    pub fn show(&self, ui: UiRef) {
        Self::padding_frame(ui, |ui| {
            ui.horizontal(|ui| self.header(ui));
            ui.separator();
        });

        self.lists(ui);
    }

    fn header_game_label(&self, ui: UiRef, game: Game) {
        let (color, tooltip) = if self.game_data[game.idx()].is_some() {
            (BLUE, "Game data loaded")
        } else {
            (
                RED,
                "Game data missing, select a valid game path in the settings",
            )
        };
        ui.label(color_text(&format!("K{game}"), color))
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
        CollapsingState::load_with_default_open(ui.ctx(), format!("save_list_{game}").into(), true)
            .show_header(ui, |ui| {
                ui.label(format!("KOTOR {game}"));
            })
            .body_unindented(|ui| self.list_inner(ui, game));
    }

    fn list_inner(&self, ui: UiRef, game: Game) {
        let game = game.idx();
        for group in &self.save_list[game] {
            ui.horizontal(|ui| {
                ui.s_offset([2., 0.]);
                ui.label(color_text("~", GREY))
                    .on_hover_text(&group.base_dir);
                if group.cloud {
                    ui.label(
                        color_text(Icon::Cloud.symbol(), GREY)
                            .size(14.)
                            .text_style(egui::TextStyle::Name("icon".into())),
                    )
                    .on_hover_text("Cloud saves");
                }
            });

            for save in &group.dirs {
                let selected = self.current_save.is_some()
                    && self.current_save.as_ref().unwrap() == &save.path;
                let label = ui.s_list_item(selected, color_text(&save.name, WHITE));
                if label.clicked() {
                    ui.ctx()
                        .send_message(Message::LoadSaveFromDir(save.path.clone()));
                }
            }
            if game != Game::COUNT - 1 {
                Self::padding_frame(ui, |ui| {
                    ui.separator();
                });
            }
        }
    }

    fn padding_frame(ui: UiRef, add_contents: impl FnOnce(UiRef)) {
        Frame::default()
            .inner_margin({
                let mut m = Margin::symmetric(8., 2.);
                m.top += 6.;
                m.right += 2.;
                m
            })
            .show(ui, add_contents);
    }
}
