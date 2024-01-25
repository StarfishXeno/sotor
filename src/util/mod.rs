pub use internal::util::*;
use internal::GameData;

mod fs;
mod ui;

pub use fs::*;
pub use ui::*;

pub fn load_default_game_data() -> [GameData; Game::COUNT] {
    let bytes = include_bytes!(concat!(env!("OUT_DIR"), "/gamedata.bin"));
    bincode::deserialize(bytes).unwrap()
}
