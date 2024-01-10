use serde::{Deserialize, Serialize};
use sotor_macros::EnumFromInt;

pub mod erf;
pub mod gff;
pub mod key;
pub mod twoda;

#[derive(Debug, PartialEq, Clone)]
pub struct FileHead {
    pub version: String,
    pub tp: String,
}

impl FileHead {
    pub fn new(version: String, tp: String) -> Self {
        Self { version, tp }
    }
}

#[repr(u16)]
#[derive(EnumFromInt, Debug, PartialEq, Eq, Clone, Hash)]
pub enum ResourceType {
    Unknown = 0,
    Txt = 10,
    Are = 2012,
    Ifo = 2014,
    Git = 2023,
    Utc = 2027,
    Fac = 2038,
    Sav = 2057,
    Tpc = 3007,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct ResourceKey(pub String, pub ResourceType);

impl From<(String, ResourceType)> for ResourceKey {
    fn from((str, tp): (String, ResourceType)) -> Self {
        Self(str, tp)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct LocString {
    pub id: u32,
    pub content: String,
}
