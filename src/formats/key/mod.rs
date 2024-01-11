use super::ResourceKey;
use std::collections::HashMap;

mod read;
pub use read::*;

#[derive(Debug, Clone, PartialEq)]
pub struct KeyResRef {
    file_idx: u32,
    resource_idx: u32,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Key {
    file_names: Vec<String>,
    resources: HashMap<ResourceKey, KeyResRef>,
}
