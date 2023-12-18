use serde::{Deserialize, Serialize};

pub mod gff;
pub mod erf;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct LocString {
    id: u32,
    content: String,
}
