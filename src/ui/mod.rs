#[cfg(not(target_arch = "wasm32"))]
use crate::util::{get_extra_save_directories, read_dir_dirs, Directory};
use crate::{
    save::Save,
    util::{load_default_game_data, ContextExt as _, Game, Message},
};
#[cfg(target_arch = "wasm32")]
use ahash::HashMap;
#[cfg(not(target_arch = "wasm32"))]
use eframe::APP_KEY;
use egui::{Context, Ui};
use internal::{GameData, GameDataMapped};
use log::error;
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};

mod editor;
#[cfg(not(target_arch = "wasm32"))]
mod settings;
#[cfg(not(target_arch = "wasm32"))]
mod side_panel;
mod styles;
mod widgets;

type UiRef<'a> = &'a mut Ui;

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug)]
struct SaveDirectories {
    cloud: bool,
    base_dir: String,
    dirs: Vec<Directory>,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
struct PersistentState {
    steam_path: Option<String>,
    game_paths: [Option<String>; Game::COUNT],
}
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug)]
pub struct SotorApp {
    save: Option<Save>,
    channel: (Sender<Message>, Receiver<Message>),
    default_game_data: [GameDataMapped; Game::COUNT],
    save_path: Option<String>,
    settings_open: bool,
    save_list: [Vec<SaveDirectories>; Game::COUNT],
    latest_save: Option<Directory>,
    game_data: [Option<GameDataMapped>; Game::COUNT],
    prs: PersistentState,
}

#[cfg(target_arch = "wasm32")]
#[derive(Debug)]
pub struct SotorApp {
    save: Option<Save>,
    channel: (Sender<Message>, Receiver<Message>),
    default_game_data: [GameDataMapped; Game::COUNT],
}

impl SotorApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        styles::set_styles(&cc.egui_ctx);
        let (sender, receiver) = channel();
        cc.egui_ctx.set_channel(sender.clone());
        let default_game_data = load_default_game_data().map(GameData::into);

        #[cfg(not(target_arch = "wasm32"))]
        {
            let mut app = Self {
                save: None,
                save_path: None,
                channel: (sender, receiver),
                default_game_data,
                settings_open: false,
                save_list: [vec![], vec![]],
                latest_save: None,
                game_data: [None, None],
                prs: cc
                    .storage
                    .and_then(|s| eframe::get_value(s, APP_KEY))
                    .unwrap_or_default(),
            };

            app.reload_save_list(&cc.egui_ctx);
            app.reload_game_data(&cc.egui_ctx);

            app
        }
        #[cfg(target_arch = "wasm32")]
        {
            Self {
                save: None,
                channel: (sender, receiver),
                default_game_data,
            }
        }
    }

    fn close_save(&mut self) {
        self.save = None;
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.save_path = None;
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl SotorApp {
    fn set_meta_id(&self, ctx: &Context) {
        let Some(save) = &self.save else {
            return;
        };

        ctx.set_meta_id(&self.default_game_data[save.game.idx()], save);
    }

    fn load_save(&mut self, files: &HashMap<String, Vec<u8>>, ctx: &Context) {
        match Save::read_from_files(files, ctx) {
            Ok(save) => {
                self.save = Some(save);
            }
            Err(err) => {
                error!("{err}");
            }
        }
        self.set_meta_id(ctx);
    }

    fn save(&mut self) {
        let bytes = Save::save_to_zip(self.save.as_mut().unwrap());
        crate::util::download_save(bytes);
    }

    fn reload_save(&mut self) {
        self.save = self.save.clone().map(Save::reload);
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl SotorApp {
    fn set_meta_id(&self, ctx: &Context) {
        let Some(save) = &self.save else {
            return;
        };
        let game_data = if let Some(data) = &self.game_data[save.game.idx()] {
            data
        } else {
            &self.default_game_data[save.game.idx()]
        };
        ctx.set_meta_id(game_data, save);
    }

    fn save(&mut self) {
        let game = self.save.as_ref().unwrap().game.idx();
        let game_data = if let Some(data) = &self.game_data[game] {
            data
        } else {
            &self.default_game_data[game]
        };
        if let Err(err) = Save::save_to_directory(
            self.save_path.as_ref().unwrap(),
            self.save.as_mut().unwrap(),
            game_data,
        ) {
            error!("{err}");
        }
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
        self.set_meta_id(ctx);
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

    fn load_game_data(&mut self, game: Game, ctx: &Context) {
        let idx = game.idx();
        if let Some(game_path) = self.prs.game_paths[idx].as_ref() {
            let game_data = GameData::read(game, game_path, self.prs.steam_path.as_ref());
            if let Ok(data) = game_data {
                self.game_data[idx] = Some(data.into());
            } else {
                self.game_data[idx] = None;
                error!("couldn't load game data for game {game}: {game_data:?}");
            }
        } else {
            self.game_data[idx] = None;
        };
        self.set_meta_id(ctx);
    }

    fn reload_game_data(&mut self, ctx: &Context) {
        Game::LIST.map(|game| self.load_game_data(game, ctx));
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
            self.load_game_data(game, ctx);
        }
        self.load_latest_save(ctx);
    }

    fn set_game_path(&mut self, game: Game, path: Option<String>, ctx: &Context) {
        self.prs.game_paths[game.idx()] = path;
        self.load_save_list(game);
        self.load_game_data(game, ctx);
        self.load_latest_save(ctx);
    }

    fn toggle_settings_open(&mut self) {
        self.settings_open = !self.settings_open;
    }
}

impl eframe::App for SotorApp {
    #[cfg(not(target_arch = "wasm32"))]
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, &self.prs);
    }

    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        while let Ok(message) = self.channel.1.try_recv() {
            #[cfg(not(target_arch = "wasm32"))]
            match message {
                Message::Save => self.save(),
                Message::CloseSave => self.close_save(),
                Message::ReloadSave => self.load_save(self.save_path.clone().unwrap(), ctx, false),
                Message::LoadSaveFromDir(path) => self.load_save(path.to_string(), ctx, false),
                Message::ToggleSettingsOpen => self.toggle_settings_open(),
                Message::SetSteamPath(path) => self.set_steam_path(path, ctx),
                Message::SetGamePath(game, path) => self.set_game_path(game, path, ctx),
                Message::ReloadSaveList => self.reload_save_list(ctx),
                Message::ReloadGameData => self.reload_game_data(ctx),
            }
            #[cfg(target_arch = "wasm32")]
            match message {
                Message::Save => self.save(),
                Message::CloseSave => self.close_save(),
                Message::ReloadSave => self.reload_save(),
                Message::LoadSaveFromFiles(files) => self.load_save(&files, ctx),
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        if self.settings_open {
            settings::Settings::new(
                || self.channel.0.send(Message::ToggleSettingsOpen).unwrap(),
                &self.prs.steam_path,
                &self.prs.game_paths,
            )
            .show(ctx);
        }

        #[cfg(not(target_arch = "wasm32"))]
        egui::SidePanel::new(egui::panel::Side::Left, "sp")
            .frame(egui::Frame::side_top_panel(&ctx.style()).inner_margin(egui::Margin::ZERO))
            .resizable(true)
            .default_width(160.)
            .min_width(160.)
            .max_width(ctx.screen_rect().width() - 760.)
            .show(ctx, |ui| {
                side_panel::SidePanel::new(&self.save_path, &self.game_data, &self.save_list)
                    .show(ui);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(save) = &mut self.save {
                #[cfg(not(target_arch = "wasm32"))]
                let current_data = if let Some(data) = &self.game_data[save.game.idx()] {
                    data
                } else {
                    &self.default_game_data[save.game.idx()]
                };
                #[cfg(target_arch = "wasm32")]
                let current_data = &self.default_game_data[save.game.idx()];
                editor::Editor::new(save, current_data).show(ui);
            } else {
                editor::editor_placeholder(ui);
            }
        });
    }
}
