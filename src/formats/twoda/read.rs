use crate::{
    formats::twoda::{TwoDA, TwoDAType},
    util::{bytes_to_string, cast_bytes, read_chunks, read_dwords, seek_to, ESResult, SResult},
};
use std::{
    collections::HashMap,
    io::{prelude::*, Cursor, SeekFrom},
};

use super::TwoDAValue;

struct Reader<'a> {
    cursor: Cursor<&'a [u8]>,
    required_columns: HashMap<String, TwoDAType>,

    target_columns: Vec<(String, usize, TwoDAType)>,
    total_columns: usize,
    row_count: usize,
    offsets: Vec<u16>,
}

impl<'a> Reader<'a> {
    fn read(mut self) -> SResult<TwoDA> {
        self.read_header()?;
        self.read_columns()?;
        self.read_row_count()?;
        self.read_row_indices()?;
        self.read_cell_offsets()?;

        self.read_rows()
    }

    fn read_header(&mut self) -> ESResult {
        let header = read_dwords(&mut self.cursor, 2).map_err(|_| "Couldn't read header")?;
        let version = bytes_to_string(&header[1].to_ne_bytes());
        if version != "V2.b" {
            return Err(format!("Invalid 2da version: {version}"));
        }
        // skip newline
        self.cursor
            .seek(SeekFrom::Current(1))
            .map_err(|_| "Couldn't seek to header ending")?;

        Ok(())
    }

    fn read_columns(&mut self) -> ESResult {
        let mut buf = vec![];
        self.cursor
            .read_until(b'\0', &mut buf)
            .map_err(|_| "Couldn't read column list")?;

        // drop the null
        buf.pop();
        // and the extra tab in the end
        buf.pop();

        let columns: Vec<_> = String::from_utf8_lossy(&buf)
            .split('\t')
            .map(ToString::to_string)
            .collect();

        self.total_columns = columns.len();
        self.target_columns = columns
            .into_iter()
            .enumerate()
            .filter_map(|(idx, col)| self.required_columns.get(&col).map(|tp| (col, idx, *tp)))
            .collect();

        if self.target_columns.len() != self.required_columns.len() {
            return Err(format!(
                "Found {} columns, required {}",
                self.target_columns.len(),
                self.required_columns.len()
            ));
        }

        Ok(())
    }

    fn read_row_count(&mut self) -> ESResult {
        let words =
            read_dwords(&mut self.cursor, 1).map_err(|_| "Couldn't read row count".to_owned())?;
        self.row_count = words[0] as usize;

        Ok(())
    }

    fn read_row_indices(&mut self) -> ESResult {
        // we don't actually care about indices, just need to seek past them
        for i in 0..self.row_count {
            let mut buf = vec![];
            self.cursor
                .read_until(b'\t', &mut buf)
                .map_err(|_| format!("Couldn't read row index {i}"))?;
        }
        Ok(())
    }

    fn read_cell_offsets(&mut self) -> ESResult {
        println!("{} {}", self.total_columns, self.row_count);
        // +1 for data size we don't need
        let mut chunks = read_chunks(&mut self.cursor, self.total_columns * self.row_count + 1, 2)
            .map_err(|_| "Couldn't read offsets".to_owned())?;
        // drop datasize
        chunks.pop();
        self.offsets = chunks.into_iter().map(|c| cast_bytes(&c)).collect();
        Ok(())
    }

    fn read_rows(&mut self) -> SResult<TwoDA> {
        let data_offset = self.cursor.position();
        let mut rows = vec![];

        for row_idx in 0..self.row_count {
            let mut row = HashMap::new();
            for (col, idx, tp) in &self.target_columns {
                seek_to!(
                    self.cursor,
                    data_offset + self.offsets[row_idx * self.total_columns + idx] as u64,
                    format
                )?;
                let mut buf = vec![];
                self.cursor
                    .read_until(b'\0', &mut buf)
                    .map_err(|_| format!("Couldn't read column {col} in row {row_idx}"))?;
                // drop the null
                buf.pop();
                let value = bytes_to_string(&buf);

                let parsed = if value.is_empty() {
                    None
                } else {
                    Some(match tp {
                        TwoDAType::Int => TwoDAValue::Int(value.parse().map_err(|_| {
                            format!(
                                "Couldn't parse int column {col} in row {row_idx}, value: {value}"
                            )
                        })?),
                        TwoDAType::String => TwoDAValue::String(value),
                    })
                };
                row.insert(col.clone(), parsed);
            }
            rows.push(row);
        }

        Ok(TwoDA(rows))
    }
}

pub fn read(bytes: &[u8], required_columns: HashMap<String, TwoDAType>) -> SResult<TwoDA> {
    let cursor = Cursor::new(bytes);
    let reader = Reader {
        cursor,
        required_columns,

        target_columns: vec![],
        total_columns: 0,
        row_count: 0,
        offsets: vec![],
    };

    reader.read().map_err(|err| format!("TwoDA::read| {err}"))
}
