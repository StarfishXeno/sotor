use crate::{
    formats::{
        impl_read_resource,
        key::{Key, KeyResRef},
        ReadResource, ResourceKey, ResourceType,
    },
    util::{
        bytes::{
            seek_to, take, take_bytes, take_head, take_string_trimmed, Cursor, IntoUsizeArray,
            DWORD_SIZE,
        },
        SResult,
    },
};
use ahash::{HashMap, HashMapExt as _};
use std::io::{BufRead, Seek, SeekFrom};

struct Header {
    file_count: usize,
    file_offset: usize,
    key_count: usize,
    key_offset: usize,
}

struct Reader<'a> {
    c: &'a mut Cursor<'a>,
}

impl<'a> Reader<'a> {
    fn new(c: &'a mut Cursor<'a>, _: ()) -> Self {
        Self { c }
    }

    fn read(mut self) -> SResult<Key> {
        let h = self.read_header()?;
        let file_data = self.read_file_data(h.file_count, h.file_offset)?;
        let file_names = self.read_file_names(file_data)?;
        let resources = self.read_resources(h.key_count, h.key_offset, &file_names)?;

        Ok(Key {
            file_names,
            resources,
        })
    }

    fn read_header(&mut self) -> SResult<Header> {
        let file_head = take_head(self.c).ok_or("couldn't read file head")?;

        if file_head.tp != "KEY " || file_head.version != "V1  " {
            return Err(format!("invalid file type or version {file_head:?}"));
        }
        let [file_count, key_count, file_offset, key_offset] = take::<[u32; 4]>(self.c)
            .ok_or("couldn't read header contents")?
            .into_usize_array();

        Ok(Header {
            file_count,
            file_offset,
            key_count,
            key_offset,
        })
    }

    fn read_file_data(&mut self, count: usize, offset: usize) -> SResult<Vec<(u32, u16)>> {
        // 2 dwords, 2 words
        const FILE_SIZE_BYTES: usize = DWORD_SIZE * 2 + 2 * 2;
        seek_to!(self.c, offset)?;
        let file_bytes =
            take_bytes(self.c, count * FILE_SIZE_BYTES).ok_or("couldn't read file table")?;
        let c = &mut Cursor::new(file_bytes);
        let mut name_data = Vec::with_capacity(count);
        for _ in 0..count {
            // don't care about file size
            c.consume(DWORD_SIZE);
            let name_offset = take::<u32>(c).unwrap();
            let name_size = take::<u16>(c).unwrap();
            // don't care about drive
            c.consume(2);

            name_data.push((name_offset, name_size));
        }

        Ok(name_data)
    }

    fn read_file_names(&mut self, data: Vec<(u32, u16)>) -> SResult<Vec<String>> {
        let mut names = Vec::with_capacity(data.len());
        for (offset, size) in data {
            seek_to!(self.c, offset)?;
            // they appear to be null-terminated
            let name = take_string_trimmed(self.c, size as usize)
                .ok_or_else(|| format!("couldn't read file name of size {size} at {offset}"))?;
            names.push(name);
        }
        Ok(names)
    }

    fn read_resources(
        &mut self,
        count: usize,
        offset: usize,
        file_names: &[String],
    ) -> SResult<HashMap<ResourceKey, KeyResRef>> {
        const RESOURCE_SIZE_BYTES: usize = 22;
        seek_to!(self.c, offset)?;
        let bytes =
            take_bytes(self.c, count * RESOURCE_SIZE_BYTES).ok_or("couldn't read key table")?;
        let c = &mut Cursor::new(bytes);
        let mut resources = HashMap::with_capacity(count);

        for _ in 0..count {
            let file_name = take_string_trimmed(c, 16).unwrap();
            let tp_raw = take::<u16>(c).unwrap();
            let id = take::<u32>(c).unwrap();
            let Ok(tp) = ResourceType::try_from(tp_raw) else {
                // we don't care about most resource types and the relevant ones are defined
                continue;
            };
            let file_idx = id >> 20;
            if file_names.get(file_idx as usize).is_none() {
                return Err(format!(
                    "resource {file_name} references invalid file index {file_idx}"
                ));
            }
            let resource_idx = id - (file_idx << 20);

            resources.insert(
                (file_name, tp).into(),
                KeyResRef {
                    file_idx,
                    resource_idx,
                },
            );
        }

        Ok(resources)
    }
}

impl_read_resource!(Key, Reader);
