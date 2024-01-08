use crate::{
    formats::{
        erf::{Erf, Resource, HEADER_SIZE, KEY_NAME_LEN, KEY_SIZE_BYTES, RESOURCE_SIZE},
        LocString, ResourceType,
    },
    util::{
        bytes_to_string, cast_bytes, read_bytes, read_dwords, read_t, seek_to, ESResult, SResult,
        ToUsizeVec as _, DWORD_SIZE,
    },
};
use std::{
    collections::HashMap,
    io::{Cursor, Seek, SeekFrom},
};

struct Header {
    file_type: String,
    file_version: String,
    loc_string_count: u32,
    entry_count: usize,
    loc_string_offset: u32,
    keys_offset: u32,
    resources_offset: u32,
    description_str_ref: u32,
}
struct KeyRead {
    name: String,
    id: u32,
    res_type: ResourceType,
}

struct ResourceRead {
    offset: usize,
    size: usize,
}

struct Reader<'a> {
    cursor: Cursor<&'a [u8]>,
    h: Header,

    loc_strings: Vec<LocString>,
    keys: Vec<KeyRead>,
    resources: Vec<ResourceRead>,
}

impl<'a> Reader<'a> {
    fn seek(&mut self, pos: u32) -> ESResult {
        seek_to!(self.cursor, pos)
    }

    fn read_header(cursor: &mut Cursor<&[u8]>) -> SResult<Header> {
        cursor.rewind().map_err(|_| "Couldn't read header")?;

        let string_bytes =
            read_t::<[u8; 4], _>(cursor, 2).map_err(|_| "Couldn't read header strings")?;
        let file_type = bytes_to_string(&string_bytes[0]);
        let file_version = bytes_to_string(&string_bytes[1]);
        let mut dwords = read_dwords(cursor, HEADER_SIZE - 2)
            .map_err(|_| "Couldn't read header data")?
            .into_iter();

        Ok(Header {
            file_type,
            file_version,

            loc_string_count: dwords.next().unwrap(),
            entry_count: {
                // don't need loc bytes
                dwords.next();
                dwords.next().unwrap() as usize
            },
            loc_string_offset: dwords.next().unwrap(),
            keys_offset: dwords.next().unwrap(),
            resources_offset: dwords.next().unwrap(),
            description_str_ref: {
                // don't need build year and day
                dwords.next();
                dwords.next();
                dwords.next().unwrap()
            },
        })
    }

    fn read_loc_strings(&mut self) -> ESResult {
        self.seek(self.h.loc_string_offset)?;
        let target_count = self.h.loc_string_count as usize;
        self.loc_strings = Vec::with_capacity(target_count);

        let mut count = 0;
        while count < target_count {
            let dwords = read_dwords(&mut self.cursor, 2)
                .map_err(|_| format!("Couldn't read LocStr head {count}"))?;
            let id = dwords[0];
            let size = dwords[1];

            let bytes = read_bytes(&mut self.cursor, size as usize)
                .map_err(|_| format!("Couldn't read LocStr {count}"))?;
            let content = bytes_to_string(&bytes);

            count += 1;
            self.loc_strings.push(LocString { id, content });
        }

        Ok(())
    }

    fn read_key_list(&mut self) -> ESResult {
        self.seek(self.h.keys_offset)?;
        let target_count = self.h.entry_count;
        self.keys = Vec::with_capacity(target_count);

        let mut count = 0;
        while count < target_count {
            let bytes = read_bytes(&mut self.cursor, KEY_SIZE_BYTES)
                .map_err(|_| format!("Couldn't read Key {count}"))?;

            let name = bytes_to_string(&bytes[..KEY_NAME_LEN])
                .trim_matches('\0')
                .to_owned();
            let id = cast_bytes(&bytes[KEY_NAME_LEN..]);
            let res_type: u16 = cast_bytes(&bytes[KEY_NAME_LEN + DWORD_SIZE..]);

            count += 1;
            self.keys.push(KeyRead {
                name,
                id,
                res_type: res_type
                    .try_into()
                    .map_err(|_| format!("Invalid resource type {res_type} in key {count}"))?,
            });
        }

        Ok(())
    }

    fn read_resources(&mut self) -> ESResult {
        self.seek(self.h.resources_offset)?;
        let target_count = self.h.entry_count;
        self.resources = Vec::with_capacity(target_count);

        let mut count = 0;
        while count < target_count {
            let dwords = read_dwords(&mut self.cursor, RESOURCE_SIZE)
                .map_err(|_| format!("Couldn't read Resource {count}"))?
                .to_usize_vec();

            count += 1;
            self.resources.push(ResourceRead {
                offset: dwords[0],
                size: dwords[1],
            });
        }

        Ok(())
    }

    fn transform(self) -> SResult<Erf> {
        let mut resources = HashMap::with_capacity(self.h.entry_count);
        let mut cursor = Cursor::new(self.cursor.into_inner());

        for (idx, key) in self.keys.into_iter().enumerate() {
            let res = &self.resources[idx];
            seek_to!(cursor, res.offset)?;

            resources.insert(
                (key.name, key.res_type),
                Resource {
                    id: key.id,
                    content: read_bytes(&mut cursor, res.size).map_err(|_| {
                        format!("Couldn't read resource content {idx} at {}", res.offset)
                    })?,
                },
            );
        }

        Ok(Erf {
            file_type: self.h.file_type,
            file_version: self.h.file_version,
            loc_strings: self.loc_strings,
            description_str_ref: self.h.description_str_ref,

            resources,
        })
    }
}

pub fn read(bytes: &[u8]) -> SResult<Erf> {
    let mut cursor = Cursor::new(bytes);
    let h = Reader::read_header(&mut cursor)?;
    let mut r = Reader {
        cursor,

        loc_strings: vec![],
        keys: vec![],
        resources: vec![],

        h,
    };
    r.read_loc_strings()?;
    r.read_key_list()?;
    r.read_resources()?;

    r.transform()
}
