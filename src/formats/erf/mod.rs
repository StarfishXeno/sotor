use crate::formats::{FileHead, LocString, ResourceKey, ResourceType};
use std::fmt;

mod read;
mod write;

use ahash::HashMap;
pub use write::*;

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
    pub file_head: FileHead,

    pub resources: HashMap<ResourceKey, Resource>,
    pub loc_strings: Vec<LocString>,
    pub description_str_ref: usize,
}

impl Erf {
    pub fn get(&self, name: &str, tp: ResourceType) -> Option<&Resource> {
        self.resources.get(&(name, tp).into())
    }

    pub fn get_mut(&mut self, name: &str, tp: ResourceType) -> Option<&mut Resource> {
        self.resources.get_mut(&(name, tp).into())
    }
}

#[cfg(test)]
mod tests {
    use crate::formats::{
        erf::{write, Erf, Resource},
        LocString, ReadResourceNoArg as _, ResourceType,
    };
    use ahash::HashMap;

    #[test]
    fn read_write() {
        let erf = Erf {
            file_head: ("TST ", "V0.0").into(),

            resources: HashMap::from_iter([
                (
                    ("pc", ResourceType::Txt).into(),
                    Resource {
                        id: 0,
                        content: (*b"pc").into(),
                    },
                ),
                (
                    ("inventory", ResourceType::Txt).into(),
                    Resource {
                        id: 1,
                        content: (*b"inventory").into(),
                    },
                ),
            ]),

            loc_strings: vec![LocString {
                id: 0,
                content: "LocString".to_owned(),
            }],
            description_str_ref: 0,
        };
        let bytes = write(erf.clone());
        let new_erf = Erf::read(&bytes).unwrap();

        assert_eq!(erf, new_erf);
    }
}
