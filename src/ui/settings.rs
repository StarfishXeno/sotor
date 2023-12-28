use egui::{Area, Context, Grid, Label, Layout, RichText, Rounding, Sense, Window};
use emath::{Align2, Pos2, Vec2};

use crate::{
    save::Game,
    util::{select_directory, ContextExt, Message},
};

use super::{
    styles::{set_button_styles, set_striped_styles, BLACK_TRANSPARENT, RED},
    widgets::{white_text, UiExt},
    UiRef,
};

pub struct Settings<'a> {
    open: &'a mut bool,
    paths: &'a mut [Option<String>; 2],
}

const WINDOW_SIZE: [f32; 2] = [400.0, 200.0];

impl<'a> Settings<'a> {
    pub fn new(open: &'a mut bool, paths: &'a mut [Option<String>; 2]) -> Self {
        Self { open, paths }
    }

    pub fn show(&mut self, ctx: &Context) {
        let res = Area::new("overlay")
            .fixed_pos(Pos2::ZERO)
            .interactable(true)
            .show(ctx, |ui| {
                let rect = ctx.screen_rect();
                let res = ui.allocate_response(rect.size(), Sense::click());

                if res.clicked() {
                    *self.open = false;
                }

                ui.painter()
                    .rect_filled(rect, Rounding::ZERO, BLACK_TRANSPARENT);
            });

        ctx.move_to_top(res.response.layer_id);

        Window::new("settings")
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .fixed_size(WINDOW_SIZE)
            .collapsible(false)
            .interactable(true)
            .title_bar(false)
            .show(ctx, |ui| {
                // fixed size is broken, this fixes it
                ui.set_width(ui.available_width());
                ui.set_height(ui.available_height());
                // this layer shit is a mess, but at least it works
                ui.with_layer_id(res.response.layer_id, |ui| {
                    Self::title_bar(ui, self.open);
                    Self::game_paths(ui, self.paths);
                });
            });
    }

    fn title_bar(ui: UiRef, open: &mut bool) {
        ui.horizontal(|ui| {
            ui.heading("Settings");
            ui.with_layout(Layout::right_to_left(emath::Align::Center), |ui| {
                let btn = ui
                    .add(
                        Label::new(RichText::new("❌").size(22.0).color(RED)).sense(Sense::click()),
                    )
                    .on_hover_cursor(egui::CursorIcon::PointingHand);
                if btn.clicked() {
                    *open = false;
                }
            });
        });
        ui.separator();
    }

    fn game_path(ui: UiRef, paths: &mut [Option<String>; 2], game: Game) {
        ui.horizontal(|ui| {
            set_button_styles(ui);
            let btn = ui.s_button_basic("Select");
            ui.label(format!("KOTOR {} path:", game as u8 + 1));

            if btn.clicked() {
                let dir = select_directory(format!(
                    "Select the directory containing {}",
                    game.get_exe_name()
                ));

                if let Some(handle) = dir {
                    let channel = ui.ctx().get_channel();
                    let path = handle.path().to_str().unwrap().to_owned();

                    channel.send(Message::SetGamePath(game, path)).unwrap();
                }
            }
        });
        ui.end_row();

        let path = paths[game as usize].as_deref().unwrap_or("None selected");
        ui.add(Label::new(white_text(path)).wrap(true));
        ui.end_row();
    }

    fn game_paths(ui: UiRef, paths: &mut [Option<String>; 2]) {
        set_striped_styles(ui);
        Grid::new("settings_paths_grid")
            .spacing([0., 5.])
            .min_col_width(ui.available_width())
            .num_columns(1)
            .striped(true)
            .show(ui, |ui| {
                Self::game_path(ui, paths, Game::One);
                Self::game_path(ui, paths, Game::Two);
            });
    }
}
