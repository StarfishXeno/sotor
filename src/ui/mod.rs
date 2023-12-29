use crate::{
    save::{Game, Save},
    util::{file_exists, read_dir_dirs, ContextExt as _, Directory, Message},
};
use eframe::APP_KEY;
use egui::{panel::Side, Context, Ui};
use log::error;
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::mpsc::{Receiver, Sender},
};

mod editor;
mod settings;
mod side_panel;
mod styles;
mod widgets;

type UiRef<'a> = &'a mut Ui;

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(default)]
struct PersistentState {
    game_paths: [Option<String>; Game::COUNT],
}

#[derive(Debug)]
pub struct SotorApp {
    save: Option<Save>,
    save_path: Option<String>,
    channel: (Sender<Message>, Receiver<Message>),
    settings_open: bool,
    save_list: [HashMap<String, Vec<Directory>>; Game::COUNT],
    latest_save: Option<Directory>,

    prs: PersistentState,
}

impl SotorApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        styles::set_styles(&cc.egui_ctx);
        let (sender, receiver) = cc.egui_ctx.set_channel();

        let mut app = Self {
            save: None,
            save_path: None,
            channel: (sender, receiver),
            settings_open: false,
            save_list: [HashMap::new(), HashMap::new()],
            latest_save: None,

            prs: cc
                .storage
                .and_then(|s| eframe::get_value(s, APP_KEY))
                .unwrap_or_default(),
        };
        app.reload_save_list();
        app.load_latest_save(&cc.egui_ctx);

        app
    }

    fn load_save(&mut self, path: String, ctx: &Context, silent: bool) {
        match Save::read_from_directory(&path, ctx) {
            Ok(save) => {
                self.save = Some(save);
                self.save_path = Some(path);
            }
            Err(err) => {
                self.save = None;
                self.save_path = None;

                if !silent {
                    error!("{err}");
                }
            }
        }
    }

    fn load_latest_save(&mut self, ctx: &Context) {
        if let Some(save) = &self.latest_save {
            self.load_save(save.path.clone(), ctx, true);
        }
    }

    fn load_save_list(&mut self, game: Game) {
        let mut saves = HashMap::new();
        let mut latest: Option<Directory> = None;

        let extra_directories = game
            .get_save_directories()
            .into_iter()
            .map(|d| ("home", d))
            .collect();

        let game_directory = self.prs.game_paths[game.to_idx()]
            .as_ref()
            .map(|path| vec![("game", PathBuf::from(path))])
            .unwrap_or_default();

        let all_paths: Vec<_> = [game_directory, extra_directories]
            .into_iter()
            .flatten()
            .flat_map(|dir| {
                let mut path = dir.1.clone();
                let mut cloud_path = dir.1;

                path.push("saves");
                cloud_path.push("cloudsaves");

                [
                    ([dir.0, "saves"].join("_"), path),
                    ([dir.0, "cloud_saves"].join("_"), cloud_path),
                ]
            })
            .collect();

        for (category, path) in all_paths {
            let mut save_dirs = vec![];
            let Ok(dirs) = read_dir_dirs(path) else {
                continue;
            };

            for dir in dirs {
                if file_exists(PathBuf::from_iter([&dir.path, "savenfo.res"])) {
                    if latest.is_none() || latest.as_ref().unwrap().date < dir.date {
                        latest = Some(dir.clone());
                    }

                    save_dirs.push(dir);
                }
            }

            save_dirs.sort_unstable_by_key(|d| d.date);
            saves.insert(category, save_dirs);
        }

        self.save_list[game.to_idx()] = saves;
        self.latest_save = latest;
    }

    fn reload_save_list(&mut self) {
        Game::LIST.map(|game| self.load_save_list(game));
    }

    fn load_game_data(&mut self, game: Game) {}

    fn reload_game_data(&mut self) {
        Game::LIST.map(|game| self.load_game_data(game));
    }

    fn set_game_path(&mut self, game: Game, path: String, ctx: &Context) {
        self.prs.game_paths[game.to_idx()] = Some(path);
        self.load_save_list(game);
        if self.save.is_none() {
            self.load_latest_save(ctx);
        }
    }
}

impl eframe::App for SotorApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, &self.prs);
    }

    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        while let Ok(message) = self.channel.1.try_recv() {
            match message {
                Message::ReloadSave => self.load_save(self.save_path.clone().unwrap(), ctx, false),
                Message::LoadFromDirectory(path) => self.load_save(path.to_string(), ctx, false),
                Message::OpenSettings => self.settings_open = true,
                Message::SetGamePath(game, path) => self.set_game_path(game, path, ctx),
                Message::ReloadSaveList => self.reload_save_list(),
                Message::ReloadGameData => self.reload_game_data(),
            }
        }

        if self.settings_open {
            settings::Settings::new(&mut self.settings_open, &mut self.prs.game_paths).show(ctx);
        }

        egui::SidePanel::new(Side::Left, "save_select")
            .resizable(true)
            .min_width(125.)
            .max_width(ctx.screen_rect().width() - 700.)
            .show(ctx, |ui| {
                side_panel::SidePanel::new(&self.prs.game_paths, &self.save_list).show(ui);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(save) = &mut self.save {
                editor::Editor::new(save).show(ui);
            } else {
                editor::editor_placeholder(ui);
            }
        });
    }
}
