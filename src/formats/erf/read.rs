use std::{
    collections::HashMap,
    io::{Cursor, Read, Seek, SeekFrom},
};

use crate::{
    formats::LocString,
    util::{bytes_to_string, read_bytes, read_dwords},
};

use super::{ERF, HEADER_SIZE};

struct Header {
    file_type: String,
    file_version: String,
    loc_string_count: u32,
    loc_string_bytes: u32,
    entry_count: u32,
    field_count: u32,
    loc_string_offset: u32,
    key_list_offset: u32,
    resource_list_offset: u32,
    build_year: u32,
    build_day: u32,
    description_str_ref: u32,
}

struct Reader<'a> {
    cursor: Cursor<&'a [u8]>,
    h: Header,

    loc_strings: Vec<LocString>,
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
        self.cursor
            .seek(SeekFrom::Start(pos as u64))
            .map(|_| ())
            .map_err(|_| rf!("Couldn't seek to {pos}"))
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
            entry_count: dwords.next().unwrap(),
            field_count: dwords.next().unwrap(),
            loc_string_offset: dwords.next().unwrap(),
            key_list_offset: dwords.next().unwrap(),
            resource_list_offset: dwords.next().unwrap(),
            build_year: dwords.next().unwrap(),
            build_day: dwords.next().unwrap(),
            description_str_ref: dwords.next().unwrap(),
        })
    }

    fn read_loc_strings(&mut self) -> RResult {
        self.seek(self.h.loc_string_offset);
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

            self.loc_strings.push(LocString { id, content });

            count += 1;
        }

        Ok(())
    }

    fn transform(self) -> ERF {
        ERF {
            file_type: self.h.file_type,
            file_version: self.h.file_version,
        }
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

        h,
    };
    r.read_loc_strings()?;
    if cfg!(debug_assertions) {
        rp!("LocStrings: {:#?}", r.loc_strings.len());
    }

    Ok(r.transform())
}
