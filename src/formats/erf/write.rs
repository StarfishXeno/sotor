use std::io::{Cursor, Seek, Write};

use crate::{util::{array_to_bytes, bytes_to_sized_bytes, get_erf_date, nullpad_string, DWORD_SIZE}, formats::ResourceType};

use super::{
    Resource, ERF, HEADER_PADDING_SIZE_BYTES, HEADER_SIZE, KEY_NAME_LEN, KEY_SIZE_BYTES,
    RESOURCE_SIZE,
};

struct KeyWrite {
    name: String,
    id: u32,
    tp: ResourceType,
}

struct ResourseWrite {
    offset: u32,
    size: u32,
}

pub struct Writer {
    file_type: String,
    file_version: String,
    description_str_ref: u32,

    loc_strings: Vec<u8>,
    loc_strings_count: u32,
    keys: Vec<KeyWrite>,
    resources: Vec<ResourseWrite>,
    data: Vec<u8>,
}

macro_rules! wf {
    ($($t:tt)*) => {{
        format!("ERF::write| {}", format!($($t)*))
    }};
}
macro_rules! wp {
    ($($t:tt)*) => {{
        println!("{}", wf!($($t)*))
    }};
}

impl Writer {
    fn new(erf: ERF) -> Self {
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
        let mut sorted: Vec<(String, Resource)> = erf.resources.into_iter().collect();
        sorted.sort_by_key(|(_, r)| r.id);

        for (name, r) in sorted {
            keys.push(KeyWrite {
                name: nullpad_string(name, KEY_NAME_LEN),
                id: r.id,
                tp: r.tp,
            });
            let offset = data.len() as u32;
            let size = r.content.len() as u32;
            data.extend(r.content);

            resources.push(ResourseWrite { offset, size })
        }

        let mut loc_strings = vec![];
        let loc_strings_count = erf.loc_strings.len() as u32;
        for str in erf.loc_strings {
            loc_strings.extend(str.id.to_le_bytes());
            loc_strings.extend(bytes_to_sized_bytes::<DWORD_SIZE>(str.content.as_bytes()));
        }

        let w = Self {
            file_type: erf.file_type,
            file_version: erf.file_version,
            description_str_ref: erf.description_str_ref,

            loc_strings,
            loc_strings_count,
            keys,
            resources,
            data,
        };

        w
    }

    fn to_bytes(self) -> Vec<u8> {
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
        let file_type = self.file_type;
        let file_version = self.file_version;
        let description_str_ref = self.description_str_ref;

        let (build_year, build_day) = get_erf_date();

        if cfg!(debug_assertions) {
            wp!(
                "Header: {} {}, built year {} day {}",
                file_type,
                file_version,
                build_year,
                build_day
            );
            wp!(
                "LocStrings: {}, {}B at {}, DescriptionStrRef: {}",
                loc_string_count,
                loc_string_bytes,
                loc_string_offset,
                description_str_ref
            );
            wp!(
                "Entries: {}, KeyList at {}, ResourceList at {}",
                entry_count,
                keys_offset,
                resources_offset,
            );
        }
        cursor.rewind().unwrap();
        cursor.write_all(file_type.as_bytes()).unwrap();
        cursor.write_all(file_version.as_bytes()).unwrap();
        cursor
            .write_all(&array_to_bytes(&[
                loc_string_count as u32,
                loc_string_bytes as u32,
                entry_count as u32,
                loc_string_offset as u32,
                keys_offset as u32,
                resources_offset as u32,
                build_year as u32,
                build_day as u32,
                description_str_ref as u32,
            ]))
            .unwrap();

        cursor.into_inner()
    }
}
pub fn write(erf: ERF) -> Vec<u8> {
    let writer = Writer::new(erf);
    writer.to_bytes()
}
