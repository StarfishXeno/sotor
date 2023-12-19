use std::mem::transmute;

use serde::{Deserialize, Serialize};

#[repr(u16)]
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum ResourceType {
    RES = 0,
    TXT = 10,
    UTC = 2027,
    FAC = 2038,
    SAV = 2057,
}

impl TryFrom<u16> for ResourceType {
    type Error = ();

    fn try_from(id: u16) -> Result<Self, Self::Error> {
        use ResourceType::*;
        // why does it have to be this way, I don't want to type out every value again
        let values = [RES, TXT, UTC, FAC, SAV].map(|v| v as u16);
        
        if values.contains(&id) {
            Ok(unsafe { transmute(id) })
        } else {
            Err(())
        }
    }
}
