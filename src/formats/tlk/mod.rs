mod read;
pub use read::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Tlk {
    language: u32,
    strings: Vec<String>,
}
