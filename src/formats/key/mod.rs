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
    resources: HashMap<ResourceKey, KeyResRef>,
}
