pub use internal::util::*;
use internal::{util::bytes::Cursor, GameData};

#[cfg(not(target_arch = "wasm32"))]
mod fs;
mod ui;

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
    let bytes = include_bytes!(concat!(env!("OUT_DIR"), "/gamedata.zip"));
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    let bin = archive.by_name("gamedata.bin").unwrap();
    bincode::deserialize_from(bin).unwrap()
}
