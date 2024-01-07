use serde::{Deserialize, Serialize};
use sotor_macros::EnumFromInt;

pub mod erf;
pub mod gff;
pub mod twoda;

#[repr(u16)]
#[derive(Debug, EnumFromInt, Serialize, Deserialize, PartialEq, Eq, Clone, Hash)]
pub enum ResourceType {
    Res = 0,
    Txt = 10,
    Are = 2012,
    Ifo = 2014,
    Git = 2023,
    Utc = 2027,
    Fac = 2038,
    Sav = 2057,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct LocString {
    pub id: u32,
    pub content: String,
}
