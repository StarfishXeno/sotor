use crate::{
    formats::{
        gff::{
            Field, FieldTmp, Gff, Orientation, Struct, Vector, FIELD_SIZE, HEADER_SIZE, STRUCT_SIZE,
        },
        LocString,
    },
    util::{
        bytes_to_string, cast_bytes, read_bytes, read_dwords, read_t, seek_to,
        sized_bytes_to_bytes, sized_bytes_to_string, ESResult, SResult, ToUsizeVec as _,
        DWORD_SIZE,
    },
};
use bytemuck::cast;
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

impl<'a> Reader<'a> {
    fn seek(&mut self, pos: usize) -> ESResult {
        seek_to!(self.cursor, pos)
    }

    fn read_header(cursor: &mut Cursor<&[u8]>) -> SResult<Header> {
        cursor.rewind().map_err(|_| "Couldn't read header")?;

        let string_bytes =
            read_t::<[u8; 4], _>(cursor, 2).map_err(|_| "Couldn't read header strings")?;
        let file_type = bytes_to_string(&string_bytes[0]);
        let file_version = bytes_to_string(&string_bytes[1]);
        let dwords =
            read_dwords(cursor, HEADER_SIZE - 2).map_err(|_| "Couldn't read header data")?;
        let mut dwords = dwords.to_usize_vec().into_iter();

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

    fn read_list_indices(&mut self) -> ESResult {
        self.seek(self.h.list_indices_offset)?;
        self.list_indices = HashMap::new();

        let target_len = self.h.list_indices_bytes / DWORD_SIZE;

        let mut dwords: &[u32] = &read_dwords(&mut self.cursor, target_len).map_err(|_| {
            format!(
                "Couldn't read list indices at starting offset {}",
                self.h.list_indices_offset
            )
        })?;
        let mut offset = 0;

        while !dwords.is_empty() {
            let size = dwords[0] as usize;
            let end = size + 1;
            let indices = &dwords[1..end];

            self.list_indices
                .insert(offset * DWORD_SIZE, indices.to_usize_vec());

            dwords = &dwords[end..];
            offset += end;
        }
        Ok(())
    }

    fn read_field_indices(&mut self) -> ESResult {
        let field_indices_offset = self.h.field_indices_offset;
        self.seek(field_indices_offset)?;

        self.field_indices = read_dwords(&mut self.cursor, self.h.field_indices_bytes / DWORD_SIZE)
            .map_err(|_| {
                format!("Couldn't read field indices, starting offset {field_indices_offset}")
            })?
            .to_usize_vec();

        Ok(())
    }

    fn read_field_data(&mut self) -> ESResult {
        self.seek(self.h.field_data_offset)?;
        self.field_data = read_bytes(&mut self.cursor, self.h.field_data_bytes).map_err(|_| {
            format!(
                "Couldn't read field data, starting offset {}",
                self.h.field_data_offset
            )
        })?;

        Ok(())
    }

    fn read_labels(&mut self) -> ESResult {
        self.seek(self.h.label_offset)?;
        self.labels = Vec::with_capacity(self.h.label_count);

        let chunks = read_t::<[u8; 16], _>(&mut self.cursor, self.h.label_count).map_err(|_| {
            format!(
                "Couldn't read field labels, starting offset {}",
                self.h.field_indices_offset
            )
        })?;

        for c in chunks {
            let label = bytes_to_string(&c).trim_matches('\0').to_owned();
            self.labels.push(label);
        }

        Ok(())
    }

    fn read_fields(&mut self) -> ESResult {
        self.seek(self.h.field_offset)?;
        self.fields = Vec::with_capacity(self.h.field_count);

        let field_data = &self.field_data;

        for i in 0..self.h.field_count {
            use FieldTmp::Simple;

            let dwords = read_dwords(&mut self.cursor, FIELD_SIZE).map_err(|_| {
                format!(
                    "Couldn't read field {i}, starting offset {}",
                    self.h.field_offset
                )
            })?;
            let label = self.labels[dwords[1] as usize].clone();
            let inner = dwords[2];
            let offset = inner as usize;
            let data = |inner_offset| {
                field_data.get(offset + inner_offset..).ok_or_else(|| {
                    format!("invalid field data offset {offset} + {inner_offset} in row {i}")
                })
            };

            let value = match dwords[0] {
                0 => Simple(Field::Byte(inner as u8)),
                1 => Simple(Field::Char(inner as i8)),
                2 => Simple(Field::Word(inner as u16)),
                3 => Simple(Field::Short(inner as i16)),
                4 => Simple(Field::Dword(inner)),
                5 => Simple(Field::Int(inner as i32)),
                6 => Simple(Field::Dword64(cast_bytes(data(0)?))),
                7 => Simple(Field::Int64(cast_bytes(data(0)?))),
                8 => Simple(Field::Float(cast(inner))),
                9 => Simple(Field::Double(cast_bytes(data(0)?))),
                10 => Simple(Field::String(sized_bytes_to_string::<u32>(data(0)?))),
                11 => Simple(Field::ResRef(sized_bytes_to_string::<u8>(data(0)?))),
                12 => {
                    let mut inner_offset = DWORD_SIZE;
                    let str_ref = cast_bytes(data(inner_offset)?);
                    inner_offset += DWORD_SIZE;
                    let count: u32 = cast_bytes(data(inner_offset)?);
                    inner_offset += DWORD_SIZE;
                    let mut strings = Vec::with_capacity(count as usize);

                    for _ in 0..count {
                        let id = cast_bytes(data(inner_offset)?);
                        inner_offset += DWORD_SIZE;
                        let content = sized_bytes_to_string::<u32>(data(inner_offset)?);
                        inner_offset += DWORD_SIZE + content.len();
                        strings.push(LocString { id, content });
                    }
                    Simple(Field::LocString(str_ref, strings))
                }
                13 => Simple(Field::Void(sized_bytes_to_bytes::<u32>(data(0)?))),
                14 => FieldTmp::Struct(offset),
                15 => {
                    let indices = self.list_indices.remove(&offset).ok_or(format!(
                        "Couldn't find list indices at {inner} in field {i}"
                    ))?;

                    FieldTmp::List(indices)
                }
                16 => {
                    const SIZE: usize = size_of::<f32>();
                    Simple(Field::Orientation(Orientation {
                        w: cast_bytes(data(0)?),
                        x: cast_bytes(data(SIZE)?),
                        y: cast_bytes(data(SIZE * 2)?),
                        z: cast_bytes(data(SIZE * 3)?),
                    }))
                }
                17 => {
                    const SIZE: usize = size_of::<f32>();

                    Simple(Field::Vector(Vector {
                        x: cast_bytes(data(0)?),
                        y: cast_bytes(data(SIZE)?),
                        z: cast_bytes(data(SIZE * 2)?),
                    }))
                }
                t => return Err(format!("Invalid field type {t} in field {i}: {label}")),
            };

            self.fields.push(FieldReadTmp { value, label });
        }
        Ok(())
    }

    fn read_structs(&mut self) -> ESResult {
        self.seek(self.h.struct_offset)?;
        self.structs = Vec::with_capacity(self.h.struct_count);

        for i in 0..self.h.struct_count {
            let dwords = read_dwords(&mut self.cursor, STRUCT_SIZE).map_err(|_| {
                format!(
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

pub fn read(bytes: &[u8]) -> SResult<Gff> {
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
