use ahash::{HashMap, HashMapExt as _};

use crate::{
    formats::{
        erf::{Erf, Resource, HEADER_SIZE, KEY_NAME_LEN, KEY_SIZE_BYTES},
        impl_read_resource, FileHead, LocString, ReadResource, ResourceType,
    },
    util::{
        bytes::{
            seek_to, take, take_bytes, take_head, take_slice, take_string, take_string_trimmed,
            Cursor, IntoUsizeVec as _,
        },
        SResult,
    },
};
use std::io::{BufRead, Seek, SeekFrom};

#[derive(Debug, Default)]
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
    c: &'a mut Cursor<'a>,
    h: Header,
}

impl<'a> Reader<'a> {
    fn new(c: &'a mut Cursor<'a>, _: ()) -> Self {
        Self {
            c,
            h: Header::default(),
        }
    }

    fn read_header(&mut self) -> SResult<Header> {
        let head = take_head(self.c).ok_or("couldn't read file head")?;
        let slice =
            take_slice::<u32>(self.c, HEADER_SIZE - 2).ok_or("couldn't read header data")?;
        let [loc_string_count, _, entry_count, loc_string_offset, keys_offset, resources_offset, _, _, description_str_ref] =
            slice.into_usize_vec().try_into().unwrap();

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
        self.h = self.read_header()?;
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
        let bytes = take_bytes(self.c, self.h.entry_count * KEY_SIZE_BYTES)
            .ok_or("couldn't read key list")?;
        let c = &mut Cursor::new(bytes);

        for idx in 0..self.h.entry_count {
            let name = take_string_trimmed(c, KEY_NAME_LEN).unwrap();
            let id = take::<u32>(c).unwrap();
            let res_type = take::<u16>(c).unwrap();
            // last 2 are unused
            c.consume(2);

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

        for [offset, size] in resource_dwords.iter() {
            resources.push(ResourceRead {
                offset: *offset as usize,
                size: *size as usize,
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
            let content = take_bytes(self.c, res.size)
                .ok_or_else(|| format!("Couldn't read resource content {idx} at {}", res.offset))?
                .to_vec();

            result.insert(
                (key.name, key.res_type).into(),
                Resource {
                    id: key.id,
                    content,
                },
            );
        }

        Ok(Erf {
            file_head: self.h.file_head,
            loc_strings,
            description_str_ref: self.h.description_str_ref,

            resources: result,
        })
    }
}

impl_read_resource!(Erf, Reader);
