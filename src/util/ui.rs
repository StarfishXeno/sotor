use egui::{ColorImage, Context, Ui};
use image::io::Reader as ImageReader;
use std::{
    any::Any,
    path::PathBuf,
    sync::mpsc::{channel, Receiver, Sender},
};

use crate::save::Game;

pub enum Message {
    ReloadSave,
    LoadFromDirectory(String),
    OpenSettings,
    SetGamePath(Game, String),
    ReloadSaveList,
    ReloadGameData,
}
pub trait ContextExt {
    fn set_channel(&self) -> (Sender<Message>, Receiver<Message>);
    fn send_message(&self, message: Message);
    fn get_data<T: 'static + Clone>(&self, id: &'static str) -> Option<T>;
    fn set_data<T: 'static + Any + Clone + Send + Sync>(&self, id: &'static str, value: T);
}
const CHANNEL_ID: &str = "sotor-channel";

impl ContextExt for Context {
    fn set_channel(&self) -> (Sender<Message>, Receiver<Message>) {
        let (sender, receiver) = channel();
        self.set_data(CHANNEL_ID, sender.clone());

        (sender, receiver)
    }

    fn send_message(&self, message: Message) {
        let channel: Sender<_> = self.get_data(CHANNEL_ID).unwrap();
        channel.send(message).unwrap();
    }

    fn get_data<T: 'static + Clone>(&self, id: &'static str) -> Option<T> {
        self.data(|data| data.get_temp(id.into()))
    }

    fn set_data<T: 'static + Any + Clone + Send + Sync>(&self, id: &'static str, value: T) {
        self.data_mut(|data| data.insert_temp(id.into(), value));
    }
}

pub fn format_seconds(secs: u32) -> String {
    let seconds = secs % 60;
    let minutes = secs / 60 % 60;
    let hours = secs / 60 / 60 % 24;
    let days = secs / 60 / 60 / 24;

    if days > 0 {
        format!("{days}d {hours}h {minutes}m {seconds}s")
    } else if hours > 0 {
        format!("{hours}h {minutes}m {seconds}s")
    } else {
        format!("{minutes}m {seconds}s")
    }
}

// something is wrong with either egui or kotor's TGAs as the normal loader fails, so have to do it this way
pub fn load_tga(path: PathBuf) -> Result<ColorImage, String> {
    let img = ImageReader::open(path)
        .map_err(|err| err.to_string())?
        .decode()
        .map_err(|err| err.to_string())?
        .brighten(10); // they're also too dark for some reason

    let size = [img.width() as _, img.height() as _];
    let rgba = img.to_rgba8();
    let flat = rgba.as_flat_samples();

    Ok(ColorImage::from_rgba_unmultiplied(size, flat.as_slice()))
}

pub struct ColumnCounter {
    max: u32,
    current: u32,
}

impl ColumnCounter {
    pub fn new(max: u32) -> Self {
        Self { max, current: 0 }
    }
    pub fn next(&mut self, ui: &mut Ui) {
        if self.current == self.max - 1 {
            self.current = 0;
            ui.end_row();
        } else {
            self.current += 1;
        }
    }
}
