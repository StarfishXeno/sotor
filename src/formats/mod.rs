use serde::{Deserialize, Serialize};

pub mod gff;
pub mod erf;
mod resource_type;

pub use resource_type::ResourceType;


#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct LocString {
    id: u32,
    content: String,
}
