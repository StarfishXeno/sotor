use crate::{
    gff::{
        FieldValue, FieldValueTmp, LocString, Struct, FIELD_SIZE, GFF, HEADER_SIZE, STRUCT_SIZE,
    },
    util::{
        bytes_to_exo_string, bytes_to_string, cast_bytes, read_bytes, read_chunks, read_dwords,
        sized_bytes_to_bytes, ToUSizeVec, DWORD_SIZE,
    },
};

use std::{
    collections::HashMap,
    io::{prelude::*, Cursor, SeekFrom},
    str::{self},
};

#[derive(Debug)]
struct FieldReadTmp {
    value: FieldValueTmp,
    label: String,
}

#[derive(Debug)]
struct StructReadTmp {
    r#type: u32,
    field_indices: Vec<usize>,
}

struct Header {
    file_type: String,
    file_version: String,

    struct_offset: usize,
    struct_count: usize,
    field_offset: usize,
    field_count: usize,
    label_offset: usize,
    label_count: usize,
    field_data_offset: usize,
    field_data_bytes: usize,
    field_indices_offset: usize,
    field_indices_bytes: usize,
    list_indices_offset: usize,
    list_indices_bytes: usize,
}

struct Reader<'a> {
    cursor: Cursor<&'a [u8]>,
    h: Header,

    list_indices: HashMap<usize, Vec<usize>>,
    field_indices: Vec<usize>,
    field_data: Vec<u8>,
    labels: Vec<String>,
    fields: Vec<FieldReadTmp>,
    structs: Vec<StructReadTmp>,
}


macro_rules! r_print {
    ($($t:tt)*) => {{
        println!("GFF::read| {}", format!($($t)*));
        println!("******************");
    }};
}

impl<'a> Reader<'a> {
    fn seek(&mut self, pos: usize) -> Result<(), String> {
        self.cursor
            .seek(SeekFrom::Start(pos as u64))
            .map(|_| ())
            .map_err(|_| format!("GFF::read| Couldn't seek to {pos}"))
    }

    fn read_header(cursor: &mut Cursor<&[u8]>) -> Result<Header, String> {
        cursor
            .seek(SeekFrom::Start(0))
            .map_err(|_| "GFF::read| Couldn't read header")?;

        let dwords =
            read_dwords(cursor, HEADER_SIZE).map_err(|_| "GFF::read| Couldn't read header")?;
        let file_type = bytes_to_string(&dwords[0].to_ne_bytes())
            .map_err(|_| "GFF::read| Couldn't parse file_type")?;
        let file_version = bytes_to_string(&dwords[1].to_ne_bytes())
            .map_err(|_| "GFF::read| Couldn't parse file_version")?;
        let mut dwords = dwords[2..].to_usize_vec().into_iter();

        Ok(Header {
            file_type,
            file_version,

            struct_offset: dwords.next().unwrap(),
            struct_count: dwords.next().unwrap(),
            field_offset: dwords.next().unwrap(),
            field_count: dwords.next().unwrap(),
            label_offset: dwords.next().unwrap(),
            label_count: dwords.next().unwrap(),
            field_data_offset: dwords.next().unwrap(),
            field_data_bytes: dwords.next().unwrap(),
            field_indices_offset: dwords.next().unwrap(),
            field_indices_bytes: dwords.next().unwrap(),
            list_indices_offset: dwords.next().unwrap(),
            list_indices_bytes: dwords.next().unwrap(),
        })
    }

    fn read_list_indices(&mut self) -> Result<(), String> {
        self.seek(self.h.list_indices_offset)?;

        let target_len = self.h.list_indices_bytes / DWORD_SIZE;

        let mut dwords: &[u32] = &read_dwords(&mut self.cursor, target_len).map_err(|_| {
            format!(
                "GFF::read| Couldn't read list indices at starting offset {}",
                self.h.list_indices_offset
            )
        })?;
        let mut offset = 0;

        loop {
            if dwords.len() == 0 {
                break;
            }
            let size = dwords[0] as usize;
            let end = size + 1;
            let indices = &dwords[1..size + 1];

            self.list_indices
                .insert(offset * DWORD_SIZE, indices.to_usize_vec());

            dwords = &dwords[end..];
            offset += end;
        }
        Ok(())
    }

    fn read_field_indices(&mut self) -> Result<(), String> {
        let field_indices_offset = self.h.field_indices_offset;
        self.seek(field_indices_offset)?;

        self.field_indices = read_dwords(&mut self.cursor, self.h.field_indices_bytes / DWORD_SIZE)
            .map_err(|_| {
                format!(
                    "GFF::read| Couldn't read field indices, starting offset {}",
                    field_indices_offset
                )
            })?
            .to_usize_vec();

        Ok(())
    }
    fn read_field_data(&mut self) -> Result<(), String> {
        self.seek(self.h.field_data_offset)?;

        self.field_data = read_bytes(&mut self.cursor, self.h.field_data_bytes).map_err(|_| {
            format!(
                "GFF::read| Couldn't read field data, starting offset {}",
                self.h.field_data_offset
            )
        })?;

        Ok(())
    }
    fn read_labels(&mut self) -> Result<(), String> {
        self.seek(self.h.label_offset)?;

        let chunks = read_chunks(&mut self.cursor, self.h.label_count, 16).map_err(|_| {
            format!(
                "GFF::read| Couldn't read field indices, starting offset {}",
                self.h.field_indices_offset
            )
        })?;
        for (idx, c) in chunks.into_iter().enumerate() {
            let label = str::from_utf8(&c)
                .map_err(|_| format!("GFF::read| Couldn't parse label {idx} {c:?}"))?
                .trim_matches('\0')
                .to_owned();
            self.labels.push(label);
        }

        Ok(())
    }

    fn read_fields(&mut self) -> Result<(), String> {
        self.seek(self.h.field_offset)?;

        for i in 0..self.h.field_count {
            use FieldValueTmp::*;
            let dwords = read_dwords(&mut self.cursor, FIELD_SIZE)
                .map_err(|_| {
                    format!(
                        "GFF::read| Couldn't read field {i}, starting offset {}",
                        self.h.field_offset
                    )
                })?
                .to_usize_vec();
            let label = self.labels[dwords[1]].clone();
            let inner = dwords[2];

            let value = match dwords[0] {
                0 => Simple(FieldValue::Byte(inner as u8)),
                1 => Simple(FieldValue::Char(inner as i8)),
                2 => Simple(FieldValue::Word(inner as u16)),
                3 => Simple(FieldValue::Short(inner as i16)),
                4 => Simple(FieldValue::Dword(inner as u32)),
                5 => Simple(FieldValue::Int(inner as i32)),
                6 => Simple(FieldValue::Dword64(cast_bytes(&self.field_data[inner..]))),
                7 => Simple(FieldValue::Int64(cast_bytes(&self.field_data[inner..]))),
                8 => Simple(FieldValue::Float(cast_bytes(&inner.to_ne_bytes()))),
                9 => Simple(FieldValue::Double(cast_bytes(&self.field_data[inner..]))),
                10 => Simple(FieldValue::CExoString(
                    bytes_to_exo_string!(&self.field_data[inner..], u32).map_err(|_| {
                        format!("GFF::read| Invalid CExoString data in field {i}: {label}")
                    })?,
                )),
                11 => Simple(FieldValue::CResRef(
                    bytes_to_exo_string!(&self.field_data[inner..], u8).map_err(|_| {
                        format!("GFF::read| Invalid CResRef data in field {i}: {label}")
                    })?,
                )),
                12 => {
                    let count: u32 = cast_bytes(&self.field_data[inner + DWORD_SIZE * 2..]);
                    let mut offset = inner + DWORD_SIZE * 3;
                    let mut strings = Vec::with_capacity(count as usize);
                    for j in 0..count {
                        let id = cast_bytes(&self.field_data[offset..]);
                        offset += DWORD_SIZE;
                        let length: u32 = cast_bytes(&self.field_data[offset..]);
                        let length = length as usize;
                        offset += DWORD_SIZE;
                        let content = bytes_to_string(&self.field_data[offset..offset + length])
                            .map_err(|_| {
                                format!(
                                    "GFF::read| Invalid CExoLocalString {j} in field {i}: {label}"
                                )
                            })?;
                        strings.push(LocString { id, content });
                        offset += length;
                    }
                    Simple(FieldValue::CExoLocString(strings))
                }
                13 => Simple(FieldValue::Void(
                    sized_bytes_to_bytes!(&self.field_data[inner..], u32).into(),
                )),
                14 => Struct(inner),
                15 => {
                    let indices = self
                        .list_indices
                        .remove(&inner)
                        .ok_or(format!("GFF::read| Couldn't find list indices {i}"))?;

                    List(indices)
                }
                t => {
                    return Err(format!(
                        "GFF::read| Invalid field type {t} in field {i}: {label}"
                    ))
                }
            };

            self.fields.push(FieldReadTmp { value, label });
        }
        Ok(())
    }
    fn read_structs(&mut self) -> Result<(), String> {
        self.seek(self.h.struct_offset)?;

        for i in 0..self.h.struct_count {
            let dwords = read_dwords(&mut self.cursor, STRUCT_SIZE).map_err(|_| {
                format!(
                    "GFF::read| Couldn't read struct {i}, starting offset {}",
                    self.h.struct_offset
                )
            })?;

            let data_or_data_offset = dwords[1] as usize;
            let field_count = dwords[2] as usize;
            let struct_field_indices = if field_count == 1 {
                vec![data_or_data_offset]
            } else {
                let start = data_or_data_offset / DWORD_SIZE;
                self.field_indices[start..start + field_count].into()
            };
            self.structs.push(StructReadTmp {
                r#type: dwords[0],
                field_indices: struct_field_indices,
            });
        }

        Ok(())
    }
    fn unwrap_tmp_field(&self, f: &FieldValueTmp) -> FieldValue {
        match f {
            FieldValueTmp::Simple(value) => value.clone(),
            FieldValueTmp::Struct(idx) => {
                FieldValue::Struct(Box::new(self.transform_struct(&self.structs[*idx])))
            }
            FieldValueTmp::List(indices) => {
                let structs: Vec<Struct> = indices
                    .into_iter()
                    .map(|idx| self.transform_struct(&self.structs[*idx]))
                    .collect();

                FieldValue::List(structs)
            }
        }
    }

    fn transform_struct(&self, s: &StructReadTmp) -> Struct {
        let mut struct_fields = HashMap::with_capacity(s.field_indices.len());
        for idx in &s.field_indices {
            let f = &self.fields[*idx];
            struct_fields.insert(f.label.clone(), self.unwrap_tmp_field(&f.value));
        }
        Struct {
            tp: s.r#type,
            fields: struct_fields,
        }
    }

    fn transform(self) -> GFF {
        GFF {
            content: self.transform_struct(&self.structs[0]),

            file_type: self.h.file_type,
            file_version: self.h.file_version,
        }
    }
}

pub fn read(bytes: &[u8]) -> Result<GFF, String> {
    let mut cursor = Cursor::new(bytes);
        let h = Reader::read_header(&mut cursor)?;
        let mut r = Reader {
            cursor,

            list_indices: HashMap::new(),
            field_indices: vec![],
            field_data: vec![],
            labels: Vec::with_capacity(h.label_count),
            fields: Vec::with_capacity(h.field_count),
            structs: Vec::with_capacity(h.struct_count),

            h,
        };
        r.read_list_indices()?;
        if cfg!(debug_assertions) {
            r_print!("List indices: {:#?}", r.list_indices.len());
        }
        r.read_field_indices()?;
        if cfg!(debug_assertions) {
            r_print!("Field indices: {:#?}", r.field_indices.len());
        }
        r.read_field_data()?;
        if cfg!(debug_assertions) {
            r_print!("Field data: {:#?}", r.field_data.len());
        }
        r.read_labels()?;
        if cfg!(debug_assertions) {
            r_print!("Labels: {:#?}", r.labels.len());
        }
        r.read_fields()?;
        if cfg!(debug_assertions) {
            r_print!("Fields: {:#?}", r.fields.len());
        }
        r.read_structs()?;
        if cfg!(debug_assertions) {
            r_print!("Structs: {}", r.structs.len());
        }
        Ok(r.transform())
}
