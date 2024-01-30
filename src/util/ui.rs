use crate::{save::Save, util::SResult};
use ahash::HashMap;
use egui::{util::id_type_map::SerializableAny, ColorImage, Context, Id, Ui};
use image::{io::Reader as ImageReader, ImageFormat};
use internal::{util::bytes::Cursor, Data, GameDataMapped};
use std::{any::Any, borrow::Cow, fmt::Display, hash::Hash, sync::mpsc::Sender};

pub enum Message {
    Save,
    CloseSave,
    ReloadSave,
    #[cfg(target_arch = "wasm32")]
    LoadSaveFromFiles(HashMap<String, Vec<u8>>),
    #[cfg(not(target_arch = "wasm32"))]
    LoadSaveFromDir(String),
    #[cfg(not(target_arch = "wasm32"))]
    ToggleSettingsOpen,
    #[cfg(not(target_arch = "wasm32"))]
    SetSteamPath(Option<String>),
    #[cfg(not(target_arch = "wasm32"))]
    SetGamePath(super::Game, Option<String>),
    #[cfg(not(target_arch = "wasm32"))]
    ReloadSaveList,
    #[cfg(not(target_arch = "wasm32"))]
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
    fn remove_data<T: 'static + Any + Clone + Send + Sync>(&self, id: impl Into<Id>);
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
        self.set_data_raw(id, (get_meta_id(self), value));
    }

    fn remove_data<T: 'static + Any + Clone + Send + Sync>(&self, id: impl Into<Id>) {
        self.data_mut(|data| data.remove_temp::<(Id, T)>(id.into()));
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
pub fn load_tga(bytes: &[u8]) -> SResult<ColorImage> {
    let mut reader = ImageReader::new(Cursor::new(bytes));
    reader.set_format(ImageFormat::Tga);
    let img = reader.decode().map_err(|err| err.to_string())?.brighten(20); // they're also too dark for some reason

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

#[cfg(not(target_arch = "wasm32"))]
pub fn load_icon() -> egui::IconData {
    let rgba = image::load_from_memory(include_bytes!("../../assets/hand.png"))
        .unwrap()
        .to_rgba8();
    let (width, height) = rgba.dimensions();
    egui::IconData {
        rgba: rgba.to_vec(),
        width,
        height,
    }
}

pub fn get_data_name<'a, I: Eq + Hash + Display, D: Data<I>>(
    data: &'a HashMap<I, D>,
    id: &I,
) -> Cow<'a, str> {
    if let Some(a) = data.get(id) {
        let n = a.get_name();
        if n.is_empty() {
            Cow::Owned(format!("UNNAMED {id}"))
        } else {
            Cow::Borrowed(n)
        }
    } else {
        Cow::Owned(format!("UNKNOWN {id}"))
    }
}
