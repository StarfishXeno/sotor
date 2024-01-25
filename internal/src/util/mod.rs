use ahash::HashMap;
use macros::EnumList;
use std::fmt::{self, Display};

pub mod bytes;
pub mod fs;

pub type SResult<T> = Result<T, String>;
pub type ESResult = SResult<()>;

#[derive(EnumList, Debug, Clone, PartialEq, Copy)]
#[repr(u8)]
pub enum Game {
    One = 0,
    Two,
}
impl Game {
    pub fn idx(self) -> usize {
        self as usize
    }
    pub fn steam_dir(self) -> &'static str {
        match self {
            Self::One => "swkotor",
            Self::Two => "Knights of the Old Republic II",
        }
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

pub fn shorten_string(str: &str, max_len: usize) -> String {
    let len = str.chars().count();
    let (final_len, elipses) = if len > max_len {
        (max_len - 3, true)
    } else {
        (len, false)
    };
    let mut string = str.chars().take(final_len).collect();
    if elipses {
        string += "...";
    }
    string
}
