use crate::{
    formats::{bif::Bif, impl_read_resource, ReadResource, ResourceType},
    util::{
        bytes::{seek_to, take, take_bytes, take_head, Cursor, DWORD_SIZE},
        SResult,
    },
};
use std::io::{BufRead, Seek, SeekFrom};

struct Reader<'a> {
    c: &'a mut Cursor<'a>,
    required_indices: &'a [usize],
}

impl<'a> Reader<'a> {
    fn new(c: &'a mut Cursor<'a>, required_indices: &'a [usize]) -> Self {
        Self {
            c,
            required_indices,
        }
    }

    fn read(mut self) -> SResult<Bif> {
        let offset = self.read_header()?;
        let resources = self.read_resources(offset)?;

        Ok(Bif { resources })
    }

    fn read_header(&mut self) -> SResult<usize> {
        let file_head = take_head(self.c).ok_or("couldn't read file head")?;

        if file_head.tp != "BIFF" || file_head.version != "V1  " {
            return Err(format!("invalid file type or version {file_head:?}"));
        }
        self.c.consume(DWORD_SIZE * 2);
        let offset = take::<u32>(self.c).ok_or("couldn't read header contents")?;

        Ok(offset as usize)
    }

    fn read_resources(&mut self, offset: usize) -> SResult<Vec<Vec<u8>>> {
        const RESOURCE_SIZE: usize = 4;
        let mut resources = Vec::with_capacity(self.required_indices.len());

        for idx in self.required_indices {
            seek_to!(self.c, offset + idx * RESOURCE_SIZE * DWORD_SIZE)?;
            // skip the id
            self.c.consume(DWORD_SIZE);
            let [offset, size] = take::<[u32; 2]>(self.c)
                .ok_or_else(|| format!("couldn't read resource {idx} offset"))?;
            seek_to!(self.c, offset)?;
            let content = take_bytes(self.c, size as usize)
                .ok_or_else(|| format!("couldn't read resource {idx} content at offset {offset}"))?
                .to_vec();

            resources.push(content);
        }

        Ok(resources)
    }
}

impl_read_resource!(Bif, Reader, &'a [usize]);
