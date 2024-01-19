use super::ResourceKey;
use std::collections::HashMap;

mod read;
pub use read::*;

#[derive(Debug, Clone, PartialEq)]
pub struct KeyResRef {
    // index into Key's file_names
    pub file_idx: u32,
    // bif resource index
    pub resource_idx: u32,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Key {
    pub file_names: Vec<String>,
    pub resources: HashMap<ResourceKey, KeyResRef>,
}
