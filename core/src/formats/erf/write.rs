use crate::{
    formats::{
        erf::{
            Erf, Resource, HEADER_PADDING_SIZE_BYTES, HEADER_SIZE, KEY_NAME_LEN, KEY_SIZE_BYTES,
            RESOURCE_SIZE,
        },
        FileHead, ResourceKey, ResourceType,
    },
    util::bytes::{bytes_to_sized_bytes, nullpad_string, IntoByteSlice as _, DWORD_SIZE},
};
use std::io::{Cursor, Seek, Write};
use time::{macros::datetime, OffsetDateTime};

struct KeyWrite {
    name: String,
    id: u32,
    tp: ResourceType,
}

struct ResourceWrite {
    offset: u32,
    size: u32,
}

pub struct Writer {
    file_head: FileHead,
    description_str_ref: u32,

    loc_strings: Vec<u8>,
    loc_strings_count: u32,
    keys: Vec<KeyWrite>,
    resources: Vec<ResourceWrite>,
    data: Vec<u8>,
}

impl Writer {
    fn new(erf: Erf) -> Self {
        let len = erf.resources.len();
        let mut keys = Vec::with_capacity(len);
        let mut resources = Vec::with_capacity(len);
        let data_len = {
            let mut len = 0;
            for r in erf.resources.values() {
                len += r.content.len();
            }
            len
        };
        let mut data = Vec::with_capacity(data_len);

        // resort it properly by ids before saving
        let mut sorted: Vec<(ResourceKey, Resource)> = erf.resources.into_iter().collect();
        sorted.sort_by_key(|(_, r)| r.id);

        for (key, r) in sorted {
            keys.push(KeyWrite {
                name: nullpad_string(r.name, KEY_NAME_LEN),
                id: r.id,
                tp: key.1,
            });
            let offset = data.len() as u32;
            let size = r.content.len() as u32;
            data.extend(r.content);

            resources.push(ResourceWrite { offset, size });
        }

        let mut loc_strings = vec![];
        let loc_strings_count = erf.loc_strings.len() as u32;
        for str in erf.loc_strings {
            loc_strings.extend(str.id.to_le_bytes());
            loc_strings.extend(bytes_to_sized_bytes::<DWORD_SIZE>(str.content.as_bytes()));
        }

        Self {
            file_head: erf.file_head,
            description_str_ref: erf.description_str_ref as u32,

            loc_strings,
            loc_strings_count,
            keys,
            resources,
            data,
        }
    }

    fn into_bytes(self) -> Vec<u8> {
        let buf = Vec::with_capacity(
            HEADER_SIZE * DWORD_SIZE
                + HEADER_PADDING_SIZE_BYTES
                + self.loc_strings.len()
                + self.keys.len() * KEY_SIZE_BYTES
                + self.resources.len() * RESOURCE_SIZE * DWORD_SIZE
                + self.data.len(),
        );
        let mut cursor = Cursor::new(buf);
        let entry_count = self.keys.len();

        // header to be filled later
        cursor
            .write_all(&[0; HEADER_SIZE * DWORD_SIZE + HEADER_PADDING_SIZE_BYTES])
            .unwrap();
        // LOCALIZED STRINGS
        let loc_string_offset = cursor.position();
        let loc_string_bytes = self.loc_strings.len();
        let loc_string_count = self.loc_strings_count;
        cursor.write_all(&self.loc_strings).unwrap();
        // KEYS
        let keys_offset = cursor.position();
        for key in self.keys {
            cursor.write_all(key.name.as_bytes()).unwrap();
            cursor.write_all(&key.id.to_le_bytes()).unwrap();
            cursor.write_all(&(key.tp as u16).to_le_bytes()).unwrap();
            cursor.write_all(&[0; 2]).unwrap();
        }
        // RESOURCES
        let resources_offset = cursor.position();
        let resources_offset_end =
            (resources_offset as usize + entry_count * RESOURCE_SIZE * DWORD_SIZE) as u32;
        for r in self.resources {
            cursor
                .write_all(&(r.offset + resources_offset_end).to_le_bytes())
                .unwrap();
            cursor.write_all(&r.size.to_le_bytes()).unwrap();
        }
        // DATA
        cursor.write_all(&self.data).unwrap();
        // HEADER
        let file_head = self.file_head;
        let description_str_ref = self.description_str_ref;

        // build date
        let now = OffsetDateTime::now_utc();
        let past = datetime!(1900 - 01 - 01 0:00 UTC);
        let build_year = now.year() - past.year();
        past.replace_year(now.year()).unwrap();
        let build_day = (now - past).whole_days();

        cursor.rewind().unwrap();
        cursor.write_all(file_head.tp.as_bytes()).unwrap();
        cursor.write_all(file_head.version.as_bytes()).unwrap();
        cursor
            .write_all(
                [
                    loc_string_count,
                    loc_string_bytes as u32,
                    entry_count as u32,
                    loc_string_offset as u32,
                    keys_offset as u32,
                    resources_offset as u32,
                    build_year as u32,
                    build_day as u32,
                    description_str_ref,
                ]
                .into_byte_slice(),
            )
            .unwrap();

        cursor.into_inner()
    }
}

pub fn write(erf: Erf) -> Vec<u8> {
    let writer = Writer::new(erf);
    writer.into_bytes()
}
