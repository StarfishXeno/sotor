use std::{
    collections::HashMap,
    io::{Cursor, Read, Seek, SeekFrom},
};

use crate::{
    formats::LocString,
    util::{bytes_to_string, cast_bytes, read_bytes, read_dwords, seek_to},
};

use super::{Resource, ERF, HEADER_SIZE, KEY_SIZE_BYTES, RESOURCE_SIZE};

struct Header {
    file_type: String,
    file_version: String,
    loc_string_count: u32,
    loc_string_bytes: u32,
    entry_count: usize,
    loc_string_offset: u32,
    key_list_offset: u32,
    resource_list_offset: u32,
    build_year: u32,
    build_day: u32,
    description_str_ref: u32,
}
struct Key {
    name: String,
    id: u32,
    res_type: u16,
}

struct ResourceReadTmp {
    offset: u32,
    size: u32,
}

struct Reader<'a> {
    cursor: Cursor<&'a [u8]>,
    h: Header,

    loc_strings: Vec<LocString>,
    keys: Vec<Key>,
    resources: Vec<ResourceReadTmp>,
}

type RResult = Result<(), String>;

macro_rules! rf {
    ($($t:tt)*) => {{
        format!("ERF::read| {}", format!($($t)*))
    }};
}

macro_rules! rp {
    ($($t:tt)*) => {{
        println!("{}", rf!($($t)*))
    }};
}

impl<'a> Reader<'a> {
    fn seek(&mut self, pos: u32) -> RResult {
        seek_to!(self.cursor, pos, rf)
    }

    fn read_header(cursor: &mut Cursor<&[u8]>) -> Result<Header, String> {
        cursor.rewind().map_err(|_| rf!("Couldn't read header"))?;

        let dwords = read_dwords(cursor, HEADER_SIZE).map_err(|_| rf!("Couldn't read header"))?;
        let file_type = bytes_to_string(&dwords[0].to_ne_bytes())
            .map_err(|_| rf!("Couldn't parse file_type"))?;
        let file_version = bytes_to_string(&dwords[1].to_ne_bytes())
            .map_err(|_| rf!("Couldn't parse file_version"))?;
        let mut dwords = dwords[2..].into_iter().copied();

        Ok(Header {
            file_type,
            file_version,

            loc_string_count: dwords.next().unwrap(),
            loc_string_bytes: dwords.next().unwrap(),
            entry_count: dwords.next().unwrap() as usize,
            loc_string_offset: dwords.next().unwrap(),
            key_list_offset: dwords.next().unwrap(),
            resource_list_offset: dwords.next().unwrap(),
            build_year: dwords.next().unwrap(),
            build_day: dwords.next().unwrap(),
            description_str_ref: dwords.next().unwrap(),
        })
    }

    fn read_loc_strings(&mut self) -> RResult {
        self.seek(self.h.loc_string_offset)?;
        let target_count = self.h.loc_string_count as usize;
        self.loc_strings = Vec::with_capacity(target_count);

        let mut count = 0;
        while count < target_count {
            let dwords = read_dwords(&mut self.cursor, 2)
                .map_err(|_| rf!("Couldn't read LocStr head {}", count))?;
            let id = dwords[0];
            let size = dwords[1];

            let bytes = read_bytes(&mut self.cursor, size as usize)
                .map_err(|_| rf!("Couldn't read LocStr {}", count))?;
            let content =
                bytes_to_string(&bytes).map_err(|_| rf!("Couldn't parse LocStr {}", count))?;

            count += 1;
            self.loc_strings.push(LocString { id, content });
        }

        Ok(())
    }

    fn read_key_list(&mut self) -> RResult {
        self.seek(self.h.key_list_offset)?;
        let target_count = self.h.entry_count;
        self.keys = Vec::with_capacity(target_count);

        let mut count = 0;
        while count < target_count {
            let bytes = read_bytes(&mut self.cursor, KEY_SIZE_BYTES)
                .map_err(|_| rf!("Couldn't read Key {}", count))?;

            let name = bytes_to_string(&bytes[..16])
                .map_err(|err| rf!("Couldn't read Key name {}, {}", count, err))?
                .trim_matches('\0')
                .to_owned();
            let id = cast_bytes(&bytes[16..]);
            let res_type = cast_bytes(&bytes[20..]);

            count += 1;
            self.keys.push(Key { name, id, res_type });
        }

        Ok(())
    }

    fn read_resources(&mut self) -> RResult {
        self.seek(self.h.resource_list_offset)?;
        let target_count = self.h.entry_count;
        self.resources = Vec::with_capacity(target_count);

        let mut count = 0;
        while count < target_count {
            let dwords = read_dwords(&mut self.cursor, RESOURCE_SIZE)
                .map_err(|_| rf!("Couldn't read Resource {}", count))?;

            count += 1;
            self.resources.push(ResourceReadTmp {
                offset: dwords[0],
                size: dwords[1],
            });
        }

        Ok(())
    }

    fn transform(self) -> Result<ERF, String> {
        let mut resources = HashMap::with_capacity(self.h.entry_count);
        let mut cursor = Cursor::new(self.cursor.into_inner());

        for (idx, key) in self.keys.into_iter().enumerate() {
            let res = &self.resources[idx];

            seek_to!(cursor, res.offset, rf)?;

            resources.insert(key.name, Resource {
                id: key.id,
                tp: key.res_type,
                content: read_bytes(&mut cursor, res.size as usize)
                    .map_err(|_| rf!("Couldn't read resource content {idx}"))?,
            });
        }

        Ok(ERF {
            file_type: self.h.file_type,
            file_version: self.h.file_version,
            loc_strings: self.loc_strings,

            resources,
        })
    }
}

pub fn read(bytes: &[u8]) -> Result<ERF, String> {
    let mut cursor = Cursor::new(bytes);
    let h = Reader::read_header(&mut cursor)?;
    if cfg!(debug_assertions) {
        rp!(
            "Header: {} {}, built year {} day {}",
            h.file_type,
            h.file_version,
            h.build_year,
            h.build_day
        );
        rp!(
            "LocStrings: {}, {}B at {}, DescriptionStrRef: {}",
            h.loc_string_count,
            h.loc_string_bytes,
            h.loc_string_offset,
            h.description_str_ref
        );
        rp!(
            "Entries: {}, KeyList at {}, ResourceList at {}",
            h.entry_count,
            h.key_list_offset,
            h.resource_list_offset,
        );
        println!("******************");
    }
    let mut r = Reader {
        cursor,

        loc_strings: vec![],
        keys: vec![],
        resources: vec![],

        h,
    };
    r.read_loc_strings()?;
    if cfg!(debug_assertions) {
        rp!("LocStrings: {:#?}", r.loc_strings.len());
    }
    r.read_key_list()?;
    if cfg!(debug_assertions) {
        rp!("Keys: {:#?}", r.keys.len());
    }
    r.read_resources()?;
    if cfg!(debug_assertions) {
        rp!("Resources: {:#?}", r.resources.len());
    }

    r.transform()
}
