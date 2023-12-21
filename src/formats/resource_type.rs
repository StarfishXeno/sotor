use serde::{Deserialize, Serialize};
use sotor_macros::EnumFromInt;

#[repr(u16)]
#[derive(Debug, EnumFromInt, Serialize, Deserialize, PartialEq, Clone)]
pub enum ResourceType {
    RES = 0,
    TXT = 10,
    ARE = 2012,
    IFO = 2014,
    GIT = 2023,
    UTC = 2027,
    FAC = 2038,
    SAV = 2057,
}


 