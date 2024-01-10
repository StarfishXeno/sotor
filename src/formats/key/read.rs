use crate::{
    formats::{
        key::{Key, KeyResRef},
        ResourceKey, ResourceType,
    },
    util::{seek_to, take, take_head, take_slice, take_string_trimmed, SResult, ToUsizeVec as _},
};
use std::{
    collections::HashMap,
    io::{Cursor, Seek, SeekFrom},
};

struct Reader<'a> {
    c: &'a mut Cursor<&'a [u8]>,
}

impl<'a> Reader<'a> {
    fn new(c: &'a mut Cursor<&'a [u8]>) -> Self {
        Self { c }
    }

    fn read(mut self) -> SResult<Key> {
        let (key_count, key_offset) = self.read_header()?;
        let resources = self.read_resources(key_count, key_offset)?;

        Ok(Key { resources })
    }

    fn read_header(&mut self) -> SResult<(usize, usize)> {
        let file_head = take_head(self.c).ok_or("couldn't read file version and type")?;

        if file_head.tp != "KEY " || file_head.version != "V1  " {
            return Err(format!("invalid file type or version {file_head:?}"));
        }
        let [_file_count, key_count, _file_offset, key_offset] = take::<[u32; 4]>(&mut self.c)
            .ok_or("could read header contents")?
            .to_usize_vec()
            .try_into()
            .unwrap();

        Ok((key_count, key_offset))
    }

    fn read_resources(
        &mut self,
        count: usize,
        offset: usize,
    ) -> SResult<HashMap<ResourceKey, KeyResRef>> {
        const RESOURCE_SIZE_BYTES: usize = 22;
        seek_to!(self.c, offset)?;
        let key_table = take_slice::<[u8; RESOURCE_SIZE_BYTES]>(&mut self.c, count)
            .ok_or("couldn't read key table")?;
        let mut resources = HashMap::with_capacity(count);

        for key in key_table {
            let c = &mut Cursor::new(key.as_ref());
            let file_name = take_string_trimmed(c, 16).unwrap();
            let tp_raw = take::<u16>(c).unwrap();
            let Ok(tp) = ResourceType::try_from(tp_raw) else {
                // we don't care about most resource types and the relevant ones are defined
                continue;
            };
            let id = take::<u32>(c).unwrap();
            let file_idx = id >> 20;
            let resource_idx = (id >> 14) - (file_idx << 6);

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

pub fn read(bytes: &[u8]) -> SResult<Key> {
    let mut c = Cursor::new(bytes);
    Reader::new(&mut c)
        .read()
        .map_err(|err| format!("Key::read| {err}"))
}
