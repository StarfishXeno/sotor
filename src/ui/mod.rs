use crate::{
    save::Save,
    util::{ContextExt as _, Message},
};
use egui::{panel::Side, Context, Ui};
use log::{error, info};
use std::sync::mpsc::Receiver;

mod editor;
mod side_panel;
mod styles;
mod widgets;

type UiRef<'a> = &'a mut Ui;

pub struct SotorApp {
    save: Option<Save>,
    save_path: Option<String>,
    channel: Receiver<Message>,
}

impl SotorApp {
    pub fn new(ctx: &eframe::CreationContext<'_>) -> Self {
        styles::set_styles(&ctx.egui_ctx);
        let receiver = ctx.egui_ctx.set_channel();

        let mut res = Self {
            save: None,
            save_path: None,
            channel: receiver,
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
}

impl eframe::App for SotorApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        while let Ok(message) = self.channel.try_recv() {
            match message {
                Message::ReloadSave => self.load_save(self.save_path.clone().unwrap(), ctx),
                Message::LoadFromDirectory(path) => self.load_save(path.to_string(), ctx),
            }
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
