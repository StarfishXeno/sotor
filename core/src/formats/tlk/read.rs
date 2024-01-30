use crate::{
    formats::{impl_read_resource, tlk::Tlk, ReadResource, ResourceType},
    util::{
        bytes::{take, take_head, take_string, Cursor, SeekExt as _, DWORD_SIZE},
        SResult,
    },
};
use std::io::BufRead as _;

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

    fn read(mut self) -> SResult<Tlk> {
        let [language, count, offset] = self.read_header()?;
        let strings = self.read_strings(count as usize, offset as usize)?;

        Ok(Tlk { language, strings })
    }

    fn read_header(&mut self) -> SResult<[u32; 3]> {
        let file_head = take_head(self.c).ok_or("couldn't read file head")?;

        if file_head.tp != "TLK " || file_head.version != "V3.0" {
            return Err(format!("invalid file type or version {file_head:?}"));
        }
        take::<[u32; 3]>(self.c).ok_or_else(|| "couldn't read header contents".to_owned())
    }

    fn read_strings(&mut self, count: usize, offset: usize) -> SResult<Vec<String>> {
        const STRING_DATA_SIZE_BYTES: usize = 10 * DWORD_SIZE;
        const HEADER_SIZE_BYTES: usize = 5 * DWORD_SIZE;
        let mut strings = Vec::with_capacity(self.required_indices.len());

        for idx in self.required_indices {
            // intentionally invalid string
            if *idx == u32::MAX as usize {
                strings.push(String::new());
                continue;
            }
            if *idx > count {
                return Err(format!(
                    "TLK contains {count} strings but index {idx} is requested"
                ));
            }
            self.c
                .seek_to(HEADER_SIZE_BYTES + idx * STRING_DATA_SIZE_BYTES)?;
            let flags =
                take::<u32>(self.c).ok_or_else(|| format!("couldn't read string {idx} flags"))?;
            // this entry has no string, meant to return an empty one
            if (flags & 1) != 1 {
                strings.push(String::new());
                continue;
            }
            self.c.consume(6 * DWORD_SIZE);
            let [str_offset, len] = take::<[u32; 2]>(self.c)
                .ok_or_else(|| format!("couldn't read string {idx} offset and size"))?;
            self.c.seek_to(offset + str_offset as usize)?;
            let content = take_string(self.c, len as usize)
                .ok_or_else(|| format!("couldn't read string {idx} content at offset {offset}"))?;

            strings.push(content);
        }

        Ok(strings)
    }
}

impl_read_resource!(Tlk, Reader, &'a [usize]);
