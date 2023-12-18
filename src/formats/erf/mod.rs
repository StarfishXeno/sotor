use std::{fmt, collections::HashMap};

use serde::{Deserialize, Serialize};
use crate::{formats::LocString, util::DWORD_SIZE};


mod read;

pub use read::read;



// 11 DWORD fields + 116 bytes reserved
const HEADER_SIZE: usize = 11;
const HEADER_PADDING_SIZE: usize = 116 / DWORD_SIZE;
// in bytes, last 2 are unused
const KEY_SIZE_BYTES: usize = 16 + 4 + 2 + 2;
// 2 DWORDs
const RESOURCE_SIZE: usize = 2;

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct Resource {
    pub id: u32,
    pub tp: u16,
    pub content: Vec<u8>,
}
// need to skip content as it's pretty big
impl fmt::Debug for Resource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Resource")
         .field("id", &self.id)
         .field("tp", &self.tp)
         .finish()
    }
}


#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ERF {
    pub file_type: String,
    pub file_version: String,

    pub resources: HashMap<String, Resource>,
    pub loc_strings: Vec<LocString>,
}

#[cfg(test)]
mod tests {}
