mod read;
pub use read::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Bif {
    resources: Vec<Vec<u8>>,
}
