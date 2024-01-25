use crate::{
    ui::{
        styles::{
            set_button_styles, set_striped_styles, BLACK, BLACK_TRANSPARENT, GREEN, RED, WHITE,
        },
        widgets::{color_text, Icon, IconButton, UiExt},
        UiRef,
    },
    util::{select_directory, ContextExt, Game, Message},
};
use egui::{Area, Context, Frame, Grid, Label, Layout, Margin, Rounding, Sense, Window};
use emath::{Align2, Pos2, Vec2};
use std::path::PathBuf;

pub struct Settings<'a, F: Fn()> {
    toggle_open: F,
    steam_path: &'a Option<String>,
    game_paths: &'a [Option<String>; 2],
}

const WINDOW_SIZE: [f32; 2] = [400., 200.];

impl<'a, F: Fn()> Settings<'a, F> {
    pub fn new(
        toggle_open: F,
        steam_path: &'a Option<String>,
        game_paths: &'a [Option<String>; 2],
    ) -> Self {
        Self {
            toggle_open,
            steam_path,
            game_paths,
        }
    }

    pub fn show(&mut self, ctx: &Context) {
        let res = Area::new("overlay")
            .fixed_pos(Pos2::ZERO)
            .interactable(true)
            .show(ctx, |ui| {
                let rect = ctx.screen_rect();

                if ui.allocate_response(rect.size(), Sense::click()).clicked() {
                    (self.toggle_open)();
                }

                ui.painter()
                    .rect_filled(rect, Rounding::ZERO, BLACK_TRANSPARENT);
            });

        ctx.move_to_top(res.response.layer_id);

        Window::new("Settings")
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

                            Self::title_bar(ui, &self.toggle_open);
                            Self::paths(ui, self.steam_path, self.game_paths);
                        });
                });
            });
    }

    fn title_bar(ui: UiRef, toggle_open: &impl Fn()) {
        ui.horizontal(|ui| {
            ui.heading("Settings");
            ui.with_layout(Layout::right_to_left(emath::Align::Center), |ui| {
                if ui.add(IconButton::new(Icon::Close).size(28.)).clicked() {
                    toggle_open();
                }
            });
        });
        ui.separator();
    }

    fn path(
        ui: UiRef,
        path: &Option<String>,
        label: &str,
        picker_title: String,
        empty_text: Option<String>,
        set_path: impl Fn(UiRef, Option<String>),
    ) {
        ui.horizontal(|ui| {
            set_button_styles(ui);

            if path.is_some() && ui.s_icon_button(Icon::Remove, "Remove").clicked() {
                set_path(ui, None);
            }
            if ui.s_button_basic("Select").clicked() {
                let dir = select_directory(picker_title);

                if let Some(handle) = dir {
                    let path = handle.path().to_str().unwrap().to_owned();
                    set_path(ui, Some(path));
                }
            }
            ui.label(label);
        });
        ui.end_row();

        let path = path.as_ref().map_or("None selected", String::as_str);
        ui.add(Label::new(color_text(path, WHITE)).wrap(true));
        ui.end_row();

        if let Some(empty_text) = empty_text {
            ui.label(color_text(&empty_text, RED));
            ui.end_row();
        }
    }

    fn paths(ui: UiRef, steam_path: &Option<String>, game_paths: &[Option<String>; 2]) {
        set_striped_styles(ui);
        Grid::new("s_grid")
            .spacing([0., 5.])
            .min_col_width(ui.available_width())
            .num_columns(1)
            .striped(true)
            .show(ui, |ui| {
                let empty_text = steam_path.as_ref().and_then(|p| {
                    (!PathBuf::from_iter([p.as_str(), "common"]).exists()).then(|| {
                        "Directory \"common\" isn't present in the selected directory".to_owned()
                    })
                });

                Self::path(
                    ui,
                    steam_path,
                    "Steam library path",
                    "Select the steamapps directory".to_owned(),
                    empty_text,
                    |ui, path| ui.ctx().send_message(Message::SetSteamPath(path)),
                );

                Game::LIST.map(|game| {
                    let exe_name = match game {
                        Game::One => "swkotor.exe",
                        Game::Two => "swkotor2.exe",
                    };
                    let path = &game_paths[game.idx()];
                    let empty_text = path.as_ref().and_then(|p| {
                        (!PathBuf::from_iter([p.as_str(), exe_name]).exists()).then(|| {
                            format!("File \"{exe_name}\" isn't present in the selected directory")
                        })
                    });

                    Self::path(
                        ui,
                        path,
                        &format!("KotOR {game} path:"),
                        format!("Select the directory containing \"{exe_name}\""),
                        empty_text,
                        |ui, path| ui.ctx().send_message(Message::SetGamePath(game, path)),
                    );
                });
            });
    }
}
