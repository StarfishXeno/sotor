use egui::{ColorImage, Context};
use image::io::Reader as ImageReader;
use std::{
    any::Any,
    sync::mpsc::{channel, Receiver, Sender},
};

pub enum Message {
    ReloadSave,
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
            data.insert_temp(CHANNEL_ID.into(), sender.clone());
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

pub fn load_tga(path: &str) -> Result<ColorImage, String> {
    let img = ImageReader::open(path)
        .map_err(|err| err.to_string())?
        .decode()
        .map_err(|err| err.to_string())?;

    let size = [img.width() as _, img.height() as _];
    let data = img.to_rgba8();
    let data = data.as_flat_samples();

    Ok(egui::ColorImage::from_rgba_unmultiplied(
        size,
        data.as_slice(),
    ))
}
