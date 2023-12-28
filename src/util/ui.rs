use egui::{ColorImage, Context};
use image::io::Reader as ImageReader;
use rfd::{AsyncFileDialog, FileHandle};
use std::{
    any::Any,
    future::Future,
    path::PathBuf,
    sync::mpsc::{channel, Receiver, Sender},
};

use crate::save::Game;

pub enum Message {
    ReloadSave,
    LoadFromDirectory(String),
    OpenSettings,
    SetGamePath(Game, String),
}
pub trait ContextExt {
    fn set_channel(&self) -> Receiver<Message>;
    fn get_channel(&self) -> Sender<Message>;
    fn get_data<T: 'static + Clone>(&self, id: &'static str) -> Option<T>;
    fn set_data<T: 'static + Any + Clone + Send + Sync>(&self, id: &'static str, value: T);
}
const CHANNEL_ID: &str = "sotor-channel";

impl ContextExt for Context {
    fn set_channel(&self) -> Receiver<Message> {
        let (sender, receiver) = channel();
        self.data_mut(|data| {
            data.insert_temp(CHANNEL_ID.into(), sender);
        });
        receiver
    }

    fn get_channel(&self) -> Sender<Message> {
        self.data(|data| data.get_temp(CHANNEL_ID.into()).unwrap())
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

pub fn select_directory(title: String) -> Option<FileHandle> {
    execute(async move { AsyncFileDialog::new().set_title(title).pick_folder().await })
}

#[cfg(not(target_arch = "wasm32"))]
fn execute<T: 'static + Send, F: Future<Output = T> + Send + 'static>(f: F) -> T {
    futures_lite::future::block_on(f)
}
#[cfg(target_arch = "wasm32")]
fn execute<T: 'static + Send, F: Future<Output = T> + Send + 'static>(f: F) -> T {
    wasm_bindgen_futures::spawn_local(f);
}
