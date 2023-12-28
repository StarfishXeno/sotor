use crate::{
    save::{Game, Save},
    util::{file_exists, read_dir_dirs, ContextExt as _, Directory, Message},
};
use egui::{
    ahash::{HashMap, HashMapExt},
    panel::Side,
    Context, Ui,
};
use log::{error, info};
use std::{path::PathBuf, sync::mpsc::Receiver};

mod editor;
mod settings;
mod side_panel;
mod styles;
mod widgets;

type UiRef<'a> = &'a mut Ui;

pub struct SotorApp {
    save: Option<Save>,
    save_path: Option<String>,
    channel: Receiver<Message>,

    settings_open: bool,
    game_paths: [Option<String>; 2],
    save_list: [HashMap<String, Vec<Directory>>; 2],
}

impl SotorApp {
    pub fn new(ctx: &eframe::CreationContext<'_>) -> Self {
        styles::set_styles(&ctx.egui_ctx);
        let receiver = ctx.egui_ctx.set_channel();

        let mut res = Self {
            save: None,
            save_path: None,
            channel: receiver,

            settings_open: false,
            game_paths: [None, None],
            save_list: [HashMap::new(), HashMap::new()],
        };

        let path = "./assets/k1/saves/000000 - QUICKSAVE".to_owned();
        res.load_save(path, &ctx.egui_ctx);

        res
    }

    fn load_save(&mut self, path: String, ctx: &Context) {
        info!("Loading save from: {path}");

        match Save::read_from_directory(&path, ctx) {
            Ok(new_save) => {
                self.save = Some(new_save);
                self.save_path = Some(path);
            }
            Err(err) => {
                self.save = None;
                self.save_path = None;

                error!("{err}");
            }
        }
    }

    fn load_save_list(&mut self, game: Game) {
        let idx = game as usize;
        let mut saves = Vec::new();
        if let Some(path) = &self.game_paths[idx] {
            let dirs = read_dir_dirs(path).unwrap();

            for dir in dirs {
                if file_exists(PathBuf::from_iter([&dir.path, "savenfo.res"])) {
                    saves.push(dir);
                }
            }
        }
        self.save_list[idx] = HashMap::from_iter([("game_dir".to_owned(), saves)]);
    }

    fn load_game_data(&mut self, game: Game, path: String) {
        self.game_paths[game as usize] = Some(path);
        self.load_save_list(game);
    }
}

impl eframe::App for SotorApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        while let Ok(message) = self.channel.try_recv() {
            match message {
                Message::ReloadSave => self.load_save(self.save_path.clone().unwrap(), ctx),
                Message::LoadFromDirectory(path) => self.load_save(path.to_string(), ctx),
                Message::OpenSettings => self.settings_open = true,
                Message::SetGamePath(game, path) => self.load_game_data(game, path),
            }
        }

        if self.settings_open {
            settings::Settings::new(&mut self.settings_open, &mut self.game_paths).show(ctx);
        }

        egui::SidePanel::new(Side::Left, "save_select")
            .resizable(true)
            .max_width(ctx.screen_rect().width() - 700.0)
            .show(ctx, |ui| side_panel::SidePanel::new().show(ui));

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(save) = &mut self.save {
                editor::Editor::new(save).show(ui);
            } else {
                editor::editor_placeholder(ui);
            };
        });
    }
}
