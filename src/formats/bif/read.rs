use crate::{
    formats::bif::Bif,
    util::{seek_to, take, take_bytes, take_head, Cursor, SResult, ToUsizeVec as _, DWORD_SIZE},
};
use std::{
    collections::HashMap,
    io::{BufRead, Seek, SeekFrom},
};

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
        let [_, _, offset] = take::<[u32; 3]>(self.c)
            .ok_or("couldn't read header contents")?
            .to_usize_vec()
            .try_into()
            .unwrap();

        Ok(offset)
    }

    fn read_resources(&mut self, offset: usize) -> SResult<HashMap<usize, Vec<u8>>> {
        const RESORUCE_SIZE: usize = 4;
        let mut resources = HashMap::with_capacity(self.required_indices.len());

        for idx in self.required_indices {
            seek_to!(self.c, offset + idx * RESORUCE_SIZE * DWORD_SIZE)?;
            // skip the id
            self.c.consume(DWORD_SIZE);
            let [offset, size] = take::<[u32; 2]>(self.c)
                .ok_or_else(|| format!("couldn't read resource {idx} offset"))?;
            seek_to!(self.c, offset)?;
            let content = take_bytes(self.c, size as usize)
                .ok_or_else(|| format!("couldn't read resource {idx} content at offset {offset}"))?
                .to_vec();

            resources.insert(*idx, content);
        }

        Ok(resources)
    }
}

pub fn read(bytes: &[u8], required_indices: &[usize]) -> SResult<Bif> {
    let c = &mut Cursor::new(bytes);
    Reader::new(c, required_indices)
        .read()
        .map_err(|err| format!("Bif::read| {err}"))
}
