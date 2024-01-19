use ahash::HashMap;
use sotor_macros::EnumList;
use std::fmt::{self, Display};

mod bytes;
mod fs;
mod ui;

pub use bytes::*;
pub use fs::*;
pub use ui::*;

pub type SResult<T> = Result<T, String>;
pub type ESResult = SResult<()>;

#[derive(Debug, EnumList, Clone, PartialEq, Copy)]
#[repr(u8)]
pub enum Game {
    One = 0,
    Two,
}
impl Game {
    pub fn idx(self) -> usize {
        self as usize
    }
}
impl Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&(*self as u8 + 1).to_string())
    }
}

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
