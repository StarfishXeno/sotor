mod read;
pub use read::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Bif {
    pub resources: Vec<Vec<u8>>,
}
