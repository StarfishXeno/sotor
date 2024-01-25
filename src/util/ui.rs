use crate::{
    save::Save,
    util::{Game, SResult},
};
use egui::{util::id_type_map::SerializableAny, ColorImage, Context, IconData, Id, Ui};
use image::io::Reader as ImageReader;
use internal::GameDataMapped;
use std::{any::Any, path::PathBuf, sync::mpsc::Sender};

pub enum Message {
    Save,
    CloseSave,
    ReloadSave,
    LoadSaveFromDir(String),
    ToggleSettingsOpen,
    SetSteamPath(Option<String>),
    SetGamePath(Game, Option<String>),
    ReloadSaveList,
    ReloadGameData,
}

pub trait ContextExt {
    fn set_channel(&self, sender: Sender<Message>);
    fn send_message(&self, message: Message);
    fn set_meta_id(&self, game_data: &GameDataMapped, save: &Save);

    fn set_data_raw<T: 'static + Any + Clone + Send + Sync>(&self, id: impl Into<Id>, value: T);
    fn get_data_raw<T: 'static + Clone>(&self, id: impl Into<Id>) -> Option<T>;
    fn get_data<T: 'static + Clone>(&self, id: impl Into<Id>) -> Option<T>;
    fn set_data<T: 'static + Any + Clone + Send + Sync>(&self, id: impl Into<Id>, value: T);
    fn get_data_prs<T: 'static + Clone + SerializableAny>(&self, id: impl Into<Id>) -> Option<T>;
    fn set_data_prs<T: 'static + Any + Clone + Send + Sync + SerializableAny>(
        &self,
        id: impl Into<Id>,
        value: T,
    );
}

const CHANNEL_ID: &str = "m_channel";
pub const META_ID_ID: &str = "m_id";

fn get_meta_id(ctx: &Context) -> Id {
    ctx.get_data_raw(META_ID_ID).unwrap()
}

impl ContextExt for Context {
    fn set_channel(&self, sender: Sender<Message>) {
        self.set_data_raw(CHANNEL_ID, sender);
    }

    fn send_message(&self, message: Message) {
        let channel: Sender<_> = self.get_data_raw(CHANNEL_ID).unwrap();
        channel.send(message).unwrap();
    }

    fn set_meta_id(&self, game_data: &GameDataMapped, save: &Save) {
        self.set_data_raw(META_ID_ID, Id::new(game_data.inner.id).with(save.id));
    }

    fn get_data_raw<T: 'static + Clone>(&self, id: impl Into<Id>) -> Option<T> {
        self.data(|data| data.get_temp(id.into()))
    }

    fn set_data_raw<T: 'static + Any + Clone + Send + Sync>(&self, id: impl Into<Id>, value: T) {
        self.data_mut(|data| data.insert_temp(id.into(), value));
    }

    // data is automatically invalidated when meta_id (i.e. game data or loaded save) changes
    fn get_data<T: 'static + Clone>(&self, id: impl Into<Id>) -> Option<T> {
        let current_meta_id = get_meta_id(self);
        if let Some((meta_id, data)) = self.get_data_raw::<(Id, T)>(id) {
            if meta_id == current_meta_id {
                Some(data)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn set_data<T: 'static + Any + Clone + Send + Sync>(&self, id: impl Into<Id>, value: T) {
        let current_meta_id = get_meta_id(self);

        self.set_data_raw(id, (current_meta_id, value));
    }

    fn get_data_prs<T: 'static + Clone + SerializableAny>(&self, id: impl Into<Id>) -> Option<T> {
        self.data_mut(|data| data.get_persisted(id.into()))
    }

    fn set_data_prs<T: 'static + Any + Clone + Send + Sync + SerializableAny>(
        &self,
        id: impl Into<Id>,
        value: T,
    ) {
        self.data_mut(|data| data.insert_persisted(id.into(), value));
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
pub fn load_tga(path: PathBuf) -> SResult<ColorImage> {
    let img = ImageReader::open(path)
        .map_err(|err| err.to_string())?
        .decode()
        .map_err(|err| err.to_string())?
        .brighten(20); // they're also too dark for some reason

    let size = [img.width() as _, img.height() as _];
    let rgba = img.into_rgba8();
    let flat = rgba.into_flat_samples();

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

pub fn load_icon() -> IconData {
    let rgba = image::load_from_memory(include_bytes!("../../assets/hand.png"))
        .unwrap()
        .to_rgba8();
    let (width, height) = rgba.dimensions();
    IconData {
        rgba: rgba.to_vec(),
        width,
        height,
    }
}
