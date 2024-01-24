use crate::{
    save::Save,
    util::{
        get_default_game_data, get_extra_save_directories, read_dir_dirs, ContextExt as _,
        Directory, Game, Message, CHANNEL_ID,
    },
};
use eframe::APP_KEY;
use egui::{panel::Side, Context, Frame, Margin, Ui};
use internal::{GameData, GameDataMapped};
use log::error;
use std::{
    path::PathBuf,
    sync::mpsc::{channel, Receiver, Sender},
};

mod editor;
mod settings;
mod side_panel;
mod styles;
mod widgets;

type UiRef<'a> = &'a mut Ui;

#[derive(Debug)]
struct SaveDirectories {
    cloud: bool,
    base_dir: String,
    dirs: Vec<Directory>,
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
struct PersistentState {
    steam_path: Option<String>,
    game_paths: [Option<String>; Game::COUNT],
}

#[derive(Debug)]
pub struct SotorApp {
    save: Option<Save>,
    save_path: Option<String>,
    channel: (Sender<Message>, Receiver<Message>),
    settings_open: bool,
    save_list: [Vec<SaveDirectories>; Game::COUNT],
    latest_save: Option<Directory>,
    game_data: [Option<GameDataMapped>; Game::COUNT],
    default_game_data: [GameDataMapped; Game::COUNT],

    prs: PersistentState,
}

impl SotorApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        styles::set_styles(&cc.egui_ctx);
        let (sender, receiver) = channel();
        cc.egui_ctx.set_data(CHANNEL_ID, sender.clone());
        let mut app = Self {
            save: None,
            save_path: None,
            channel: (sender, receiver),
            settings_open: false,
            save_list: [vec![], vec![]],
            latest_save: None,
            game_data: [None, None],
            default_game_data: get_default_game_data().map(GameData::into),

            prs: cc
                .storage
                .and_then(|s| eframe::get_value(s, APP_KEY))
                .unwrap_or_default(),
        };

        app.reload_save_list(&cc.egui_ctx);
        app.reload_game_data();

        app
    }

    fn save(&mut self) {
        if let Err(err) = Save::save_to_directory(
            self.save_path.as_ref().unwrap(),
            self.save.as_mut().unwrap(),
        ) {
            error!("{err}");
        }
    }

    fn close_save(&mut self) {
        self.save = None;
        self.save_path = None;
    }

    fn load_save(&mut self, path: String, ctx: &Context, silent: bool) {
        match Save::read_from_directory(&path, ctx) {
            Ok(save) => {
                self.save = Some(save);
                self.save_path = Some(path);
            }
            Err(err) => {
                if !silent {
                    error!("{err}");
                }
            }
        }
    }

    fn load_latest_save(&mut self, ctx: &Context) {
        if self.save.is_some() {
            return;
        }
        if let Some(save) = &self.latest_save {
            self.load_save(save.path.clone(), ctx, true);
        }
    }

    fn load_save_list(&mut self, game: Game) {
        let mut saves = vec![];
        let mut latest: Option<Directory> = None;

        let extra_directories = get_extra_save_directories(game);

        let game_directory = self.prs.game_paths[game.idx()]
            .as_ref()
            .map(|path| vec![PathBuf::from(path)])
            .unwrap_or_default();

        let all_paths: Vec<_> = [game_directory, extra_directories]
            .into_iter()
            .flatten()
            .flat_map(|dir| {
                let base_dir: String = dir.to_string_lossy().into();
                let mut path = dir.clone();
                path.push("saves");

                let mut cloud_path = dir;
                cloud_path.push("cloudsaves");
                let cloud_dirs = read_dir_dirs(cloud_path)
                    .unwrap_or_default()
                    .into_iter()
                    .map(|d| (true, base_dir.clone(), d.path.into()))
                    .collect();

                [vec![(false, base_dir.clone(), path)], cloud_dirs].concat()
            })
            .collect();

        for (cloud, base_dir, path) in all_paths {
            let mut save_dirs = vec![];
            let Ok(dirs) = read_dir_dirs(path) else {
                continue;
            };

            for dir in dirs {
                if PathBuf::from_iter([&dir.path, "savenfo.res"]).exists() {
                    if latest.is_none() || latest.as_ref().unwrap().date < dir.date {
                        latest = Some(dir.clone());
                    }

                    save_dirs.push(dir);
                }
            }

            if !save_dirs.is_empty() {
                save_dirs.sort_unstable_by(|a, b| b.date.cmp(&a.date));
                saves.push(SaveDirectories {
                    cloud,
                    base_dir,
                    dirs: save_dirs,
                });
            }
        }
        saves.sort_unstable_by(|a, b| b.dirs[0].date.cmp(&a.dirs[0].date));
        self.save_list[game.idx()] = saves;
        if let Some(dir) = latest {
            if self.latest_save.is_none() || self.latest_save.as_ref().unwrap().date < dir.date {
                self.latest_save = Some(dir);
            }
        }
    }

    fn reload_save_list(&mut self, ctx: &Context) {
        Game::LIST.map(|game| self.load_save_list(game));

        if self.save.is_none() {
            self.load_latest_save(ctx);
        }
    }

    fn load_game_data(&mut self, game: Game) {
        let idx = game.idx();
        let Some(game_path) = self.prs.game_paths[idx].as_ref() else {
            self.game_data[idx] = None;
            return;
        };
        let game_data = GameData::read(game, game_path, self.prs.steam_path.as_ref());
        if let Ok(data) = game_data {
            self.game_data[idx] = Some(data.into());
        } else {
            error!("couldn't load game data for game {game}: {game_data:?}");
        }
    }

    fn reload_game_data(&mut self) {
        Game::LIST.map(|game| self.load_game_data(game));
    }

    fn set_steam_path(&mut self, path: Option<String>, ctx: &Context) {
        self.prs.steam_path = path;
        let Some(path) = self.prs.steam_path.clone() else {
            return;
        };
        // set game paths if they aren't selected yet
        for game in Game::LIST {
            let game_path = &mut self.prs.game_paths[game.idx()];
            if game_path.is_some() {
                continue;
            };
            let game_dir = game.steam_dir();
            let new_path = PathBuf::from_iter([path.as_str(), "common", game_dir]);
            if new_path.exists() {
                *game_path = Some(new_path.to_str().unwrap().to_owned());
            }
            self.load_save_list(game);
            self.load_game_data(game);
        }
        self.load_latest_save(ctx);
    }
    fn set_game_path(&mut self, game: Game, path: Option<String>, ctx: &Context) {
        self.prs.game_paths[game.idx()] = path;
        self.load_save_list(game);
        self.load_game_data(game);
        self.load_latest_save(ctx);
    }
}

impl eframe::App for SotorApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, &self.prs);
    }

    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        while let Ok(message) = self.channel.1.try_recv() {
            match message {
                Message::Save => self.save(),
                Message::CloseSave => self.close_save(),
                Message::ReloadSave => self.load_save(self.save_path.clone().unwrap(), ctx, false),
                Message::LoadSaveFromDir(path) => self.load_save(path.to_string(), ctx, false),
                Message::OpenSettings => self.settings_open = true,
                Message::SetSteamPath(path) => self.set_steam_path(path, ctx),
                Message::SetGamePath(game, path) => self.set_game_path(game, path, ctx),
                Message::ReloadSaveList => self.reload_save_list(ctx),
                Message::ReloadGameData => self.reload_game_data(),
            }
        }

        if self.settings_open {
            settings::Settings::new(
                &mut self.settings_open,
                &self.prs.steam_path,
                &self.prs.game_paths,
            )
            .show(ctx);
        }

        egui::SidePanel::new(Side::Left, "save_select")
            .frame(Frame::side_top_panel(&ctx.style()).inner_margin(Margin::ZERO))
            .resizable(true)
            .min_width(150.)
            .max_width(ctx.screen_rect().width() - 700.)
            .show(ctx, |ui| {
                side_panel::SidePanel::new(&self.save_path, &self.game_data, &self.save_list)
                    .show(ui);
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
