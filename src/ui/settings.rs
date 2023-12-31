use std::path::PathBuf;

use egui::{Area, Context, Frame, Grid, Label, Layout, Margin, Rounding, Sense, Window};
use emath::{Align2, Pos2, Vec2};

use crate::{
    save::Game,
    util::{select_directory, ContextExt, Message},
};

use super::{
    styles::{set_button_styles, set_striped_styles, BLACK, BLACK_TRANSPARENT, GREEN, RED, WHITE},
    widgets::{color_text, Icon, IconButton, UiExt},
    UiRef,
};

pub struct Settings<'a> {
    open: &'a mut bool,
    paths: &'a mut [Option<String>; 2],
}

const WINDOW_SIZE: [f32; 2] = [400., 200.];

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
            .frame(
                Frame::default()
                    .inner_margin(Margin::ZERO)
                    .outer_margin(Margin::ZERO)
                    .shadow(ctx.style().visuals.window_shadow),
            )
            .show(ctx, |ui| {
                // this layer shit is a mess, but at least it works
                ui.with_layer_id(res.response.layer_id, |ui| {
                    Frame::default()
                        .stroke((2., GREEN))
                        .inner_margin(6.)
                        .fill(BLACK)
                        .rounding(5.)
                        .show(ui, |ui| {
                            // fixed size is broken, this fixes it
                            ui.set_width(ui.available_width());
                            ui.set_height(ui.available_height());

                            Self::title_bar(ui, self.open);
                            Self::game_paths(ui, self.paths);
                        });
                });
            });
    }

    fn title_bar(ui: UiRef, open: &mut bool) {
        ui.horizontal(|ui| {
            ui.heading("Settings");
            ui.with_layout(Layout::right_to_left(emath::Align::Center), |ui| {
                let btn = ui.add(IconButton::new(Icon::Close).size(28.));

                if btn.clicked() {
                    *open = false;
                }
            });
        });
        ui.separator();
    }

    fn game_path(ui: UiRef, paths: &mut [Option<String>; 2], game: Game) {
        let exe_name = match game {
            Game::One => "swkotor.exe".to_owned(),
            Game::Two => "swkotor2.exe".to_owned(),
        };
        ui.horizontal(|ui| {
            set_button_styles(ui);
            let btn = ui.s_button_basic("Select");
            ui.label(format!("KOTOR {game} path:"));

            if btn.clicked() {
                let dir = select_directory(format!("Select the directory containing {exe_name}",));

                if let Some(handle) = dir {
                    let path = handle.path().to_str().unwrap().to_owned();

                    ui.ctx().send_message(Message::SetGamePath(game, path));
                }
            }
        });
        ui.end_row();

        let path = paths[game.idx()].as_deref().unwrap_or("None selected");
        ui.add(Label::new(color_text(path, WHITE)).wrap(true));
        ui.end_row();

        if !PathBuf::from_iter([path, &exe_name]).exists() {
            ui.label(color_text(
                &format!("{exe_name} isn't present in the selected directory"),
                RED,
            ));
        }
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
                Game::LIST.map(|game| Self::game_path(ui, paths, game));
            });
    }
}
