mod read;
pub use read::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Tlk {
    pub language: u32,
    pub strings: Vec<String>,
}
