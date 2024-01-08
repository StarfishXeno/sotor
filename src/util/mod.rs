use std::collections::HashMap;

mod bytes;
mod fs;
mod ui;

pub use bytes::*;
pub use fs::*;
pub use ui::*;

use crate::save::Game;

pub type SResult<T> = Result<T, String>;
pub type ESResult = SResult<()>;

pub fn string_lowercase_map(strings: &[String]) -> HashMap<String, String> {
    strings
        .iter()
        .map(|s| (s.to_lowercase(), s.to_string()))
        .collect()
}

const PARTY_1: &[&str] = &[
    "Bastila",
    "Canderous",
    "Carth",
    "HK-47",
    "Jolee",
    "Juhani",
    "Mission",
    "T3-M4",
    "Zaalbar",
];

const PARTY_2: &[&str] = &[
    "Atton",
    "Bao-Dur",
    "Mandalore",
    "G0-T0",
    "Handmaiden",
    "HK-47",
    "Kreia",
    "Mira",
    "T3-M4",
    "Visas",
    "Hanharr",
    "Disciple",
];

pub fn get_party(game: Game) -> &'static [&'static str] {
    match game {
        Game::One => PARTY_1,
        Game::Two => PARTY_2,
    }
}
