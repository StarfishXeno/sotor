use serde::{Deserialize, Serialize};
use sotor_macros::EnumFromInt;

#[repr(u16)]
#[derive(Debug, EnumFromInt, Serialize, Deserialize, PartialEq, Clone)]
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
