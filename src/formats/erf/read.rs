use crate::{
    formats::{
        erf::{Erf, Resource, KEY_NAME_LEN, KEY_SIZE_BYTES},
        FileHead, LocString, ResourceType,
    },
    util::{
        seek_to, take, take_head, take_slice, take_string, take_string_trimmed, SResult,
        ToUsizeVec as _,
    },
};
use std::{
    collections::HashMap,
    io::{Cursor, Seek, SeekFrom},
};

use super::HEADER_SIZE;

#[derive(Debug)]
struct Header {
    file_head: FileHead,
    loc_string_count: usize,
    entry_count: usize,
    loc_string_offset: usize,
    keys_offset: usize,
    resources_offset: usize,
    description_str_ref: usize,
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
    c: &'a mut Cursor<&'a [u8]>,
    h: Header,
}

impl<'a> Reader<'a> {
    fn new(c: &'a mut Cursor<&'a [u8]>) -> SResult<Self> {
        let h = Self::read_header(c)?;
        Ok(Self { c, h })
    }

    fn read_header(c: &mut Cursor<&[u8]>) -> SResult<Header> {
        let head = take_head(c).ok_or("Couldn't read head")?;
        let slice = take_slice::<u32>(c, HEADER_SIZE - 2).ok_or("couldn't read header data")?;
        let [loc_string_count, _, entry_count, loc_string_offset, keys_offset, resources_offset, _, _, description_str_ref] =
            slice.to_usize_vec().try_into().unwrap();

        Ok(Header {
            file_head: head,

            loc_string_count,
            entry_count,
            loc_string_offset,
            keys_offset,
            resources_offset,
            description_str_ref,
        })
    }

    fn read(mut self) -> SResult<Erf> {
        let loc_strings = self.read_loc_strings()?;
        let key_list = self.read_key_list()?;
        let resources = self.read_resources()?;

        self.transform(key_list, &resources, loc_strings)
    }

    fn read_loc_strings(&mut self) -> SResult<Vec<LocString>> {
        seek_to!(self.c, self.h.loc_string_offset)?;
        let target_count = self.h.loc_string_count;
        let mut loc_strings = Vec::with_capacity(target_count);

        let mut count = 0;
        while count < target_count {
            let [id, len] = take::<[u32; 2]>(self.c)
                .ok_or_else(|| format!("Couldn't read LocStr head {count}"))?;

            let content = take_string(self.c, len as usize)
                .ok_or_else(|| format!("Couldn't read LocStr {count}"))?;

            count += 1;
            loc_strings.push(LocString { id, content });
        }

        Ok(loc_strings)
    }

    fn read_key_list(&mut self) -> SResult<Vec<KeyRead>> {
        seek_to!(self.c, self.h.keys_offset)?;
        let mut keys = Vec::with_capacity(self.h.entry_count);
        let key_bytes = take_slice::<[u8; KEY_SIZE_BYTES]>(self.c, self.h.entry_count)
            .ok_or("couldn't read key list")?;

        for (idx, bytes) in key_bytes.iter().enumerate() {
            let c = &mut Cursor::new(bytes.as_ref());
            let name = take_string_trimmed(c, KEY_NAME_LEN).unwrap();
            let id = take::<u32>(c).unwrap();
            let res_type = take::<u16>(c).unwrap();

            keys.push(KeyRead {
                name,
                id,
                res_type: res_type
                    .try_into()
                    .map_err(|_| format!("invalid resource type {res_type} in key {idx}"))?,
            });
        }

        Ok(keys)
    }

    fn read_resources(&mut self) -> SResult<Vec<ResourceRead>> {
        seek_to!(self.c, self.h.resources_offset)?;
        let mut resources = Vec::with_capacity(self.h.entry_count);
        let resource_dwords =
            take_slice::<[u32; 2]>(self.c, self.h.entry_count).ok_or("couldn't read resources")?;

        for [offset, size] in resource_dwords {
            resources.push(ResourceRead {
                offset: offset as usize,
                size: size as usize,
            });
        }

        Ok(resources)
    }

    fn transform(
        self,
        keys: Vec<KeyRead>,
        resources: &[ResourceRead],
        loc_strings: Vec<LocString>,
    ) -> SResult<Erf> {
        let mut result = HashMap::with_capacity(self.h.entry_count);

        for (idx, key) in keys.into_iter().enumerate() {
            let res = &resources[idx];
            seek_to!(self.c, res.offset)?;

            let resource = Resource {
                id: key.id,
                content: take_slice::<u8>(self.c, res.size).ok_or_else(|| {
                    format!("Couldn't read resource content {idx} at {}", res.offset)
                })?,
            };
            result.insert((key.name, key.res_type).into(), resource);
        }

        Ok(Erf {
            file_head: self.h.file_head,
            loc_strings,
            description_str_ref: self.h.description_str_ref,

            resources: result,
        })
    }
}

pub fn read(bytes: &[u8]) -> SResult<Erf> {
    let c = &mut Cursor::new(bytes);
    Reader::new(c)
        .and_then(Reader::read)
        .map_err(|err| format!("Erf::read| {err}"))
}
