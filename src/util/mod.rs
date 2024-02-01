use crate::save::{Character, Nfo};
pub use core::util::*;
use core::{util::bytes::Cursor, GameData};

#[cfg(not(target_arch = "wasm32"))]
mod fs;
mod ui;

pub static ASSETS: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/assets.zip"));

#[cfg(target_arch = "wasm32")]
mod fs {
    use std::future::Future;
    use wasm_bindgen::prelude::*;

    pub async fn select_save() -> Option<Vec<rfd::FileHandle>> {
        rfd::AsyncFileDialog::new()
            .set_title("Select all files from a save directory")
            .pick_files()
            .await
    }

    pub fn execute<F: Future<Output = ()> + 'static>(f: F) {
        wasm_bindgen_futures::spawn_local(f);
    }

    #[wasm_bindgen]
    extern "C" {
        pub fn download_save(blob: Vec<u8>);
    }
}

pub use fs::*;
pub use ui::*;

pub fn load_default_game_data() -> [GameData; Game::COUNT] {
    let mut archive = zip::ZipArchive::new(Cursor::new(ASSETS)).unwrap();
    let bin = archive.by_name("gamedata.bin").unwrap();
    bincode::deserialize_from(bin).unwrap()
}

// current hp/fp value doesn't include a bunch of bonus modifiers and is therefore lower than the actual value
// this attempts to guess the difference
// still doesn't account for gear attribute bonuses and such, since there's no practical way to calculate them
pub fn calc_hp_fp_offset(char: &Character) -> (i16, i16) {
    let mut level = 0;
    let mut force_level = 0;
    for class in &char.classes {
        level += class.level;
        if class.powers.is_some() {
            force_level += class.level;
        }
    }

    let mut hp = 0;
    let mut toughness = 0;
    let mut wookiee_toughness = 0;
    for id in char.feats.iter().copied() {
        // toughness and master toughness
        if id == 84 || id == 124 {
            toughness += 1;
        }
        if id == 95 || id == 224 || id == 225 {
            wookiee_toughness += 1;
        }
        // war veteran
        if id == 206 {
            hp += 25;
        }
    }
    // they go 2 > 3 > 4
    if wookiee_toughness > 0 {
        wookiee_toughness += 1;
    }

    // first level's hp, doesn't apply half the time because reasons
    // let first_level = char
    //     .classes
    //     .first()
    //     .and_then(|c| data.classes.get(&c.id))
    //     .map(|c| c.hit_die)
    //     .unwrap_or_default() as i16;

    // constitution
    let constitution = (char.attributes[2] as i16 - 10) / 2;

    hp += (toughness + wookiee_toughness) * level;
    hp += constitution * level;

    let mut fp = 0;
    let wisdom = (char.attributes[4] as i16 - 10) / 2;
    fp += wisdom * force_level;

    (hp, fp)
}

pub fn find_pc_name<'a>(chars: &'a [Character], nfo: &'a Nfo) -> &'a str {
    chars
        .iter()
        .find(|c| c.tag.is_empty())
        .map(|c| c.name.as_str())
        .or(nfo.pc_name.as_deref())
        .unwrap_or("UNKNOWN")
}
