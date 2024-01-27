pub use internal::util::*;
use internal::GameData;

#[cfg(not(target_arch = "wasm32"))]
mod fs;
mod ui;

#[cfg(target_arch = "wasm32")]
mod fs {
    use wasm_bindgen::prelude::*;
    pub async fn select_save() -> Option<Vec<rfd::FileHandle>> {
        rfd::AsyncFileDialog::new()
            .set_title("Select all files from the save directory")
            .pick_files()
            .await
    }

    use std::future::Future;
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
    let bytes = include_bytes!(concat!(env!("OUT_DIR"), "/gamedata.bin"));
    bincode::deserialize(bytes).unwrap()
}
