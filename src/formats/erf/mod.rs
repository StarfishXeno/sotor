use std::{collections::HashMap, fmt};

use crate::formats::LocString;

mod read;
mod write;

pub use read::read;
pub use write::write;

use super::ResourceType;

// 11 DWORD fields + 116 bytes reserved
const HEADER_SIZE: usize = 11;
const HEADER_PADDING_SIZE_BYTES: usize = 116;

const KEY_NAME_LEN: usize = 16;
// in bytes, last 2 bytes are unused
const KEY_SIZE_BYTES: usize = KEY_NAME_LEN + 4 + 2 + 2;
// 2 DWORDs
const RESOURCE_SIZE: usize = 2;

#[derive(PartialEq, Clone)]
pub struct Resource {
    pub id: u32,
    pub tp: ResourceType,
    pub content: Vec<u8>,
}
// need to skip content as it's pretty big
impl fmt::Debug for Resource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Resource")
            .field("id", &self.id)
            .field("tp", &self.tp)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Erf {
    pub file_type: String,
    pub file_version: String,
    pub build_year: u32,
    pub build_day: u32,

    pub resources: HashMap<String, Resource>,
    pub loc_strings: Vec<LocString>,
    pub description_str_ref: u32,
}

#[cfg(test)]
mod tests {
    use crate::{
        formats::{
            erf::{read, write, Erf, Resource},
            LocString, ResourceType,
        },
        util::get_erf_date,
    };

    #[test]
    fn read_write() {
        let (build_year, build_day) = get_erf_date();

        let erf = Erf {
            file_type: "TST ".to_owned(),
            file_version: "V0.0".to_owned(),
            build_year,
            build_day,

            resources: [
                (
                    "pc".to_owned(),
                    Resource {
                        id: 0,
                        tp: ResourceType::Txt,
                        content: (*b"pc").into(),
                    },
                ),
                (
                    "inventory".to_owned(),
                    Resource {
                        id: 1,
                        tp: ResourceType::Txt,
                        content: (*b"inventory").into(),
                    },
                ),
            ]
            .into(),

            loc_strings: vec![LocString {
                id: 0,
                content: "LocString".to_owned(),
            }],
            description_str_ref: 0,
        };
        let bytes = write(erf.clone());
        let new_erf = read(&bytes).unwrap();

        assert_eq!(erf, new_erf);
    }
}
