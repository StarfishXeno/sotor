use std::collections::HashMap;

mod read;
pub use read::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Bif {
    resources: HashMap<usize, Vec<u8>>,
}
