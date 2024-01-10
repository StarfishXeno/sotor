use crate::{
    formats::twoda::{TwoDA, TwoDAType, TwoDAValue},
    util::{seek_to, take, take_head, take_slice, take_string_until, ESResult, SResult},
};
use std::{
    collections::HashMap,
    io::{prelude::*, Cursor, SeekFrom},
    usize,
};

struct TargetColumn {
    name: String,
    idx: usize,
    tp: TwoDAType,
}

struct Reader<'a> {
    c: Cursor<&'a [u8]>,
    required_columns: HashMap<String, TwoDAType>,
}

impl<'a> Reader<'a> {
    fn read(mut self) -> SResult<TwoDA> {
        self.read_header()?;
        let (total_columns, target_columns) = self.read_columns()?;
        let row_count = self.read_row_count()? as usize;
        let row_indices = self.read_row_indices(row_count)?;
        let cell_offsets = self.read_cell_offsets(total_columns, row_count)?;

        self.read_rows(
            row_count,
            total_columns,
            &row_indices,
            &target_columns,
            &cell_offsets,
        )
    }

    fn read_header(&mut self) -> ESResult {
        let file_head = take_head(&mut self.c).ok_or("couldn't read header")?;
        if file_head.version != "V2.b" {
            return Err(format!("Invalid 2da version: {}", file_head.version));
        }
        // skip newline
        self.c.consume(1);

        Ok(())
    }

    fn read_columns(&mut self) -> SResult<(usize, Vec<TargetColumn>)> {
        let mut columns_str =
            take_string_until(&mut self.c, b'\0').ok_or("couldn't read column list")?;
        // drop the extra tab in the end
        columns_str.pop();
        let columns: Vec<_> = columns_str.split('\t').map(ToString::to_string).collect();

        let total_columns = columns.len();
        let target_columns: Vec<_> = columns
            .into_iter()
            .enumerate()
            .filter_map(|(idx, name)| {
                self.required_columns
                    .get(&name)
                    .map(|tp| TargetColumn { name, idx, tp: *tp })
            })
            .collect();

        if target_columns.len() != self.required_columns.len() {
            return Err(format!(
                "found {} columns, required {}",
                target_columns.len(),
                self.required_columns.len()
            ));
        }

        Ok((total_columns, target_columns))
    }

    fn read_row_count(&mut self) -> SResult<u32> {
        take::<u32>(&mut self.c).ok_or("couldn't read row count".to_owned())
    }

    fn read_row_indices(&mut self, row_count: usize) -> SResult<Vec<usize>> {
        let mut row_indices = vec![];
        for i in 0..row_count {
            let string = take_string_until(&mut self.c, b'\t')
                .ok_or_else(|| format!("couldn't read row index {i}"))?;

            row_indices.push(
                string
                    .parse()
                    .map_err(|_| format!("invalid row index {string}"))?,
            );
        }
        Ok(row_indices)
    }

    fn read_cell_offsets(&mut self, total_columns: usize, row_count: usize) -> SResult<Vec<u16>> {
        // +1 for data size we don't need
        let mut offsets = take_slice::<u16>(&mut self.c, total_columns * row_count + 1)
            .ok_or("couldn't read offsets")?;
        // drop datasize
        offsets.pop();

        Ok(offsets)
    }

    fn read_rows(
        &mut self,
        row_count: usize,
        total_columns: usize,
        row_indices: &[usize],
        target_columns: &[TargetColumn],
        offsets: &[u16],
    ) -> SResult<TwoDA> {
        let data_offset = self.c.position();
        let mut rows = vec![];

        for row_idx in 0..row_count {
            let provided_idx = row_indices.get(row_idx).copied().unwrap_or_default();
            if row_idx != provided_idx {
                return Err(format!(
                    "row index {provided_idx} doesn't match actual position {row_idx}"
                ));
            }

            rows.push(self.read_row(
                data_offset,
                row_idx,
                total_columns,
                target_columns,
                offsets,
            )?);
        }

        Ok(TwoDA(rows))
    }

    fn read_row(
        &mut self,
        data_offset: u64,
        row_idx: usize,
        total_columns: usize,
        target_columns: &[TargetColumn],
        offsets: &[u16],
    ) -> SResult<HashMap<String, Option<TwoDAValue>>> {
        let mut row = HashMap::new();
        for TargetColumn { name, idx, tp } in target_columns {
            seek_to!(
                self.c,
                data_offset + offsets[row_idx * total_columns + idx] as u64
            )?;
            let value = take_string_until(&mut self.c, b'\0')
                .ok_or_else(|| format!("couldn't read column {name} in row {row_idx}"))?;

            let parsed = if value.is_empty() {
                None
            } else {
                Some(match tp {
                    TwoDAType::Int => TwoDAValue::Int(value.parse().map_err(|_| {
                        format!("couldn't parse int column {name} in row {row_idx}, value: {value}")
                    })?),
                    TwoDAType::String => TwoDAValue::String(value),
                })
            };
            row.insert(name.clone(), parsed);
        }
        Ok(row)
    }
}

pub fn read(bytes: &[u8], required_columns: HashMap<String, TwoDAType>) -> SResult<TwoDA> {
    let cursor = Cursor::new(bytes);
    let reader = Reader {
        c: cursor,
        required_columns,
    };

    reader.read().map_err(|err| format!("TwoDA::read| {err}"))
}
