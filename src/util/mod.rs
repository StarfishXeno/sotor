mod bytes;
mod fs;
mod ui;

use std::collections::HashMap;

pub use bytes::*;
pub use fs::*;
pub use ui::*;

pub fn string_lowercase_map(strings: &[String]) -> HashMap<String, String> {
    strings
        .iter()
        .map(|s| (s.to_lowercase(), s.to_string()))
        .collect()
}
