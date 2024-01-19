use super::ResourceKey;
use ahash::HashMap;
use std::path::PathBuf;

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

impl Key {
    pub fn get_file_path(&self, idx: u32) -> PathBuf {
        let file_name = &self.file_names[idx as usize];
        file_name.split('\\').collect()
    }
}
