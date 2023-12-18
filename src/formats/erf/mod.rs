use std::collections::HashMap;

use serde::{Deserialize, Serialize};

mod read;

pub use read::read;

use crate::util::DWORD_SIZE;

// 11 DWORD fields + 116 bytes reserved 
const HEADER_SIZE: usize = 11;
const HEADER_RESERVED_SIZE: usize = 116 / DWORD_SIZE;



#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ERF {
    pub file_type: String,
    pub file_version: String,
}

#[cfg(test)]
mod tests {
}
