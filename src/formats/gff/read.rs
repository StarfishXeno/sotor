use super::{Field, FieldTmp, Gff, Struct, FIELD_SIZE, HEADER_SIZE, STRUCT_SIZE};
use crate::{
    formats::{
        gff::{Orientation, Vector},
        LocString,
    },
    util::{
        bytes_to_exo_string, bytes_to_string, cast_bytes, read_chunks, read_dwords, seek_to,
        sized_bytes_to_bytes, ToUSizeVec, DWORD_SIZE,
    },
};
use std::{
    collections::HashMap,
    io::{prelude::*, Cursor, SeekFrom},
    mem::size_of,
};

#[derive(Debug)]
struct FieldReadTmp {
    value: FieldTmp,
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

type RResult = Result<(), String>;

macro_rules! rf {
    ($($t:tt)*) => {{
        format!("GFF::read| {}", format!($($t)*))
    }};
}

impl<'a> Reader<'a> {
    fn seek(&mut self, pos: usize) -> RResult {
        seek_to!(self.cursor, pos, rf)
    }

    fn read_header(cursor: &mut Cursor<&[u8]>) -> Result<Header, String> {
        cursor.rewind().map_err(|_| rf!("Couldn't read header"))?;

        let dwords = read_dwords(cursor, HEADER_SIZE).map_err(|_| rf!("Couldn't read header"))?;
        let file_type = bytes_to_string(&dwords[0].to_ne_bytes());
        let file_version = bytes_to_string(&dwords[1].to_ne_bytes());
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

    fn read_list_indices(&mut self) -> RResult {
        self.seek(self.h.list_indices_offset)?;
        self.list_indices = HashMap::new();

        let target_len = self.h.list_indices_bytes / DWORD_SIZE;

        let mut dwords: &[u32] = &read_dwords(&mut self.cursor, target_len).map_err(|_| {
            rf!(
                "Couldn't read list indices at starting offset {}",
                self.h.list_indices_offset
            )
        })?;
        let mut offset = 0;

        while !dwords.is_empty() {
            let size = dwords[0] as usize;
            let end = size + 1;
            let indices = &dwords[1..=size];

            self.list_indices
                .insert(offset * DWORD_SIZE, indices.to_usize_vec());

            dwords = &dwords[end..];
            offset += end;
        }
        Ok(())
    }

    fn read_field_indices(&mut self) -> RResult {
        let field_indices_offset = self.h.field_indices_offset;
        self.seek(field_indices_offset)?;

        self.field_indices = read_dwords(&mut self.cursor, self.h.field_indices_bytes / DWORD_SIZE)
            .map_err(|_| {
                rf!(
                    "Couldn't read field indices, starting offset {}",
                    field_indices_offset
                )
            })?
            .to_usize_vec();

        Ok(())
    }

    fn read_field_data(&mut self) -> RResult {
        self.seek(self.h.field_data_offset)?;
        self.field_data = vec![0; self.h.field_data_bytes];

        self.cursor.read_exact(&mut self.field_data).map_err(|_| {
            rf!(
                "Couldn't read field data, starting offset {}",
                self.h.field_data_offset
            )
        })?;

        Ok(())
    }

    fn read_labels(&mut self) -> RResult {
        self.seek(self.h.label_offset)?;
        self.labels = Vec::with_capacity(self.h.label_count);

        let chunks = read_chunks(&mut self.cursor, self.h.label_count, 16).map_err(|_| {
            rf!(
                "Couldn't read field indices, starting offset {}",
                self.h.field_indices_offset
            )
        })?;
        for c in chunks {
            let label = String::from_utf8_lossy(&c).trim_matches('\0').to_owned();
            self.labels.push(label);
        }

        Ok(())
    }

    fn read_fields(&mut self) -> RResult {
        self.seek(self.h.field_offset)?;
        self.fields = Vec::with_capacity(self.h.field_count);

        let field_data = &self.field_data;

        for i in 0..self.h.field_count {
            use FieldTmp::Simple;

            let dwords = read_dwords(&mut self.cursor, FIELD_SIZE)
                .map_err(|_| {
                    rf!(
                        "Couldn't read field {i}, starting offset {}",
                        self.h.field_offset
                    )
                })?
                .to_usize_vec();
            let label = self.labels[dwords[1]].clone();
            let inner = dwords[2];

            let value = match dwords[0] {
                0 => Simple(Field::Byte(inner as u8)),
                1 => Simple(Field::Char(inner as i8)),
                2 => Simple(Field::Word(inner as u16)),
                3 => Simple(Field::Short(inner as i16)),
                4 => Simple(Field::Dword(inner as u32)),
                5 => Simple(Field::Int(inner as i32)),
                6 => Simple(Field::Dword64(cast_bytes(&field_data[inner..]))),
                7 => Simple(Field::Int64(cast_bytes(&field_data[inner..]))),
                8 => Simple(Field::Float(cast_bytes(&inner.to_ne_bytes()))),
                9 => Simple(Field::Double(cast_bytes(&field_data[inner..]))),
                10 => Simple(Field::String(bytes_to_exo_string!(
                    &field_data[inner..],
                    u32
                ))),
                11 => Simple(Field::ResRef(bytes_to_exo_string!(
                    &field_data[inner..],
                    u8
                ))),
                12 => {
                    let str_ref: u32 = cast_bytes(&field_data[inner + DWORD_SIZE..]);
                    let count: u32 = cast_bytes(&field_data[inner + DWORD_SIZE * 2..]);
                    let mut offset = inner + DWORD_SIZE * 3;
                    let mut strings = Vec::with_capacity(count as usize);
                    for _ in 0..count {
                        let id = cast_bytes(&field_data[offset..]);
                        offset += DWORD_SIZE;
                        let length: u32 = cast_bytes(&field_data[offset..]);
                        let length = length as usize;
                        offset += DWORD_SIZE;
                        let content = bytes_to_string(&field_data[offset..offset + length]);
                        strings.push(LocString { id, content });
                        offset += length;
                    }
                    Simple(Field::LocString(str_ref, strings))
                }
                13 => Simple(Field::Void(
                    sized_bytes_to_bytes!(&field_data[inner..], u32).into(),
                )),
                14 => FieldTmp::Struct(inner),
                15 => {
                    let indices = self
                        .list_indices
                        .remove(&inner)
                        .ok_or(rf!("Couldn't find list indices at {inner} in field {i}"))?;

                    FieldTmp::List(indices)
                }
                16 => {
                    const SIZE: usize = size_of::<f32>();
                    let bytes = &field_data[inner..];

                    Simple(Field::Orientation(Orientation {
                        w: cast_bytes(&bytes[0..]),
                        x: cast_bytes(&bytes[SIZE..]),
                        y: cast_bytes(&bytes[SIZE * 2..]),
                        z: cast_bytes(&bytes[SIZE * 3..]),
                    }))
                }
                17 => {
                    const SIZE: usize = size_of::<f32>();
                    let bytes = &field_data[inner..];

                    Simple(Field::Vector(Vector {
                        x: cast_bytes(&bytes[0..]),
                        y: cast_bytes(&bytes[SIZE..]),
                        z: cast_bytes(&bytes[SIZE * 2..]),
                    }))
                }
                t => return Err(rf!("Invalid field type {t} in field {i}: {label}")),
            };

            self.fields.push(FieldReadTmp { value, label });
        }
        Ok(())
    }

    fn read_structs(&mut self) -> RResult {
        self.seek(self.h.struct_offset)?;
        self.structs = Vec::with_capacity(self.h.struct_count);

        for i in 0..self.h.struct_count {
            let dwords = read_dwords(&mut self.cursor, STRUCT_SIZE).map_err(|_| {
                rf!(
                    "Couldn't read struct {i}, starting offset {}",
                    self.h.struct_offset
                )
            })?;

            let data_or_data_offset = dwords[1] as usize;
            let field_count = dwords[2] as usize;
            let struct_field_indices = match field_count {
                0 => vec![],
                1 => vec![data_or_data_offset],
                _ => {
                    let start = data_or_data_offset / DWORD_SIZE;
                    self.field_indices[start..start + field_count].into()
                }
            };
            self.structs.push(StructReadTmp {
                r#type: dwords[0],
                field_indices: struct_field_indices,
            });
        }

        Ok(())
    }

    fn unwrap_tmp_field(&self, f: &FieldTmp) -> Field {
        match f {
            FieldTmp::Simple(value) => value.clone(),
            FieldTmp::Struct(idx) => {
                Field::Struct(Box::new(self.transform_struct(&self.structs[*idx])))
            }
            FieldTmp::List(indices) => {
                let structs: Vec<Struct> = indices
                    .iter()
                    .map(|idx| self.transform_struct(&self.structs[*idx]))
                    .collect();

                Field::List(structs)
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

    fn transform(self) -> Gff {
        Gff {
            content: self.transform_struct(&self.structs[0]),

            file_type: self.h.file_type,
            file_version: self.h.file_version,
        }
    }
}

pub fn read(bytes: &[u8]) -> Result<Gff, String> {
    let mut cursor = Cursor::new(bytes);
    let h = Reader::read_header(&mut cursor)?;
    let mut r = Reader {
        cursor,

        list_indices: HashMap::new(),
        field_indices: vec![],
        field_data: vec![],
        labels: vec![],
        fields: vec![],
        structs: vec![],

        h,
    };
    r.read_list_indices()?;
    r.read_field_indices()?;
    r.read_field_data()?;
    r.read_labels()?;
    r.read_fields()?;
    r.read_structs()?;

    Ok(r.transform())
}
