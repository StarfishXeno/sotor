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
    pub content: Vec<u8>,
}
// need to skip content as it's pretty big
impl fmt::Debug for Resource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Resource")
            .field("id", &self.id)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Erf {
    pub file_type: String,
    pub file_version: String,

    pub resources: HashMap<(String, ResourceType), Resource>,
    pub loc_strings: Vec<LocString>,
    pub description_str_ref: u32,
}

#[cfg(test)]
mod tests {
    use crate::formats::{
        erf::{read, write, Erf, Resource},
        LocString, ResourceType,
    };

    #[test]
    fn read_write() {
        let erf = Erf {
            file_type: "TST ".to_owned(),
            file_version: "V0.0".to_owned(),

            resources: [
                (
                    ("pc".to_owned(), ResourceType::Txt),
                    Resource {
                        id: 0,
                        content: (*b"pc").into(),
                    },
                ),
                (
                    ("inventory".to_owned(), ResourceType::Txt),
                    Resource {
                        id: 1,
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
