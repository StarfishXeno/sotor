use crate::{
    formats::{
        gff::{
            Field, FieldTmp, Gff, Orientation, Struct, Vector, FIELD_SIZE, HEADER_SIZE, STRUCT_SIZE,
        },
        FileHead, LocString,
    },
    util::{
        seek_to, take, take_head, take_slice, take_slice_sized, take_string_sized,
        take_string_trimmed, ESResult, SResult, ToUsizeVec as _, DWORD_SIZE,
    },
};
use bytemuck::cast;
use std::{
    collections::HashMap,
    io::{prelude::*, Cursor, SeekFrom},
};

#[derive(Debug)]
struct FieldReadTmp {
    value: FieldTmp,
    label: String,
}

#[derive(Debug)]
struct StructReadTmp {
    tp: u32,
    field_indices: Vec<usize>,
}

struct Header {
    file_head: FileHead,

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
    c: &'a mut Cursor<&'a [u8]>,
    h: Header,

    list_indices: HashMap<usize, Vec<usize>>,
    field_indices: Vec<u32>,
    field_data: &'a [u8],
    labels: Vec<String>,
    fields: Vec<FieldReadTmp>,
    structs: Vec<StructReadTmp>,
}

impl<'a> Reader<'a> {
    fn new(c: &'a mut Cursor<&'a [u8]>) -> SResult<Self> {
        let h = Self::read_header(c)?;
        Ok(Self {
            c,
            h,

            list_indices: HashMap::new(),
            field_indices: vec![],
            field_data: &[],
            labels: vec![],
            fields: vec![],
            structs: vec![],
        })
    }

    fn read(mut self) -> SResult<Gff> {
        self.read_list_indices()?;
        self.read_field_indices()?;
        self.read_field_data()?;
        self.read_labels()?;
        self.read_fields()?;
        self.read_structs()?;

        Ok(Gff {
            content: self.transform_struct(&self.structs[0]),

            file_head: self.h.file_head,
        })
    }

    fn read_header(c: &mut Cursor<&[u8]>) -> SResult<Header> {
        let file_head = take_head(c).ok_or("couldn't read header strings")?;
        let dwords = take_slice::<u32>(c, HEADER_SIZE - 2).ok_or("couldn't read header data")?;
        let mut dwords = dwords.to_usize_vec().into_iter();

        Ok(Header {
            file_head,

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
        seek_to!(self.c, self.h.list_indices_offset)?;
        self.list_indices = HashMap::new();

        let bytes = take_slice::<u8>(self.c, self.h.list_indices_bytes).ok_or_else(|| {
            format!(
                "Couldn't read list indices at starting offset {}",
                self.h.list_indices_offset
            )
        })?;
        let c = &mut Cursor::new(bytes.as_ref());
        let mut offset = 0;

        while offset < self.h.list_indices_bytes {
            let size = take::<u32>(c)
                .ok_or_else(|| format!("couldn't read list index size at {offset}"))?;
            let indices = take_slice::<u32>(c, size as usize)
                .ok_or_else(|| format!("couldn't read {size} list indices at {offset}"))?;

            self.list_indices.insert(offset, indices.to_usize_vec());

            offset = c.position() as usize;
        }
        Ok(())
    }

    fn read_field_indices(&mut self) -> ESResult {
        let offset = self.h.field_indices_offset;
        seek_to!(self.c, offset)?;

        self.field_indices = take_slice::<u32>(self.c, self.h.field_indices_bytes / DWORD_SIZE)
            .ok_or_else(|| format!("Couldn't read field indices, starting offset {offset}"))?;

        Ok(())
    }

    fn read_field_data(&mut self) -> ESResult {
        let offset = self.h.field_data_offset;
        self.field_data = self
            .c
            .get_ref()
            .get(offset..offset + self.h.field_data_bytes)
            .ok_or("couldn't read field data")?;

        Ok(())
    }

    fn read_labels(&mut self) -> ESResult {
        seek_to!(self.c, self.h.label_offset)?;
        self.labels = Vec::with_capacity(self.h.label_count);

        for idx in 0..self.h.label_count {
            let label = take_string_trimmed(self.c, 16)
                .ok_or_else(|| format!("couldn't read label {idx}"))?;
            self.labels.push(label);
        }

        Ok(())
    }

    fn read_fields(&mut self) -> ESResult {
        seek_to!(self.c, self.h.field_offset)?;
        self.fields = Vec::with_capacity(self.h.field_count);
        let field_data = self.field_data;
        let fields = take_slice::<[u32; FIELD_SIZE]>(self.c, self.h.field_count)
            .ok_or("couldn't read fields")?;

        for (idx, [tp, label_idx, value]) in fields.iter().copied().enumerate() {
            use FieldTmp::Simple;
            let label = self
                .labels
                .get(label_idx as usize)
                .ok_or_else(|| format!("missing label {label_idx} in field {idx}"))?
                .clone();

            let value = match tp {
                0 => Simple(Field::Byte(value as u8)),
                1 => Simple(Field::Char(value as i8)),
                2 => Simple(Field::Word(value as u16)),
                3 => Simple(Field::Short(value as i16)),
                4 => Simple(Field::Dword(value)),
                5 => Simple(Field::Int(value as i32)),
                6 => Simple(read_data(tp, field_data, value, idx, |c| {
                    take(c).map(Field::Dword64)
                })?),
                7 => Simple(read_data(tp, field_data, value, idx, |c| {
                    take(c).map(Field::Int64)
                })?),
                8 => Simple(Field::Float(cast(value))),
                9 => Simple(read_data(tp, field_data, value, idx, |c| {
                    take(c).map(Field::Double)
                })?),
                10 => Simple(read_data(tp, field_data, value, idx, |c| {
                    take_string_sized::<u32>(c).map(Field::String)
                })?),
                11 => Simple(read_data(tp, field_data, value, idx, |c| {
                    take_string_sized::<u8>(c).map(Field::ResRef)
                })?),
                12 => Simple(read_data(tp, field_data, value, idx, |c| {
                    c.consume(DWORD_SIZE);
                    let [str_ref, count] = take::<[u32; 2]>(c)?;
                    let mut strings = Vec::with_capacity(count as usize);

                    for _ in 0..count {
                        let id = take::<u32>(c)?;
                        let content = take_string_sized::<u32>(c)?;
                        strings.push(LocString { id, content });
                    }
                    Some(Field::LocString(str_ref, strings))
                })?),
                13 => Simple(read_data(tp, field_data, value, idx, |c| {
                    take_slice_sized::<u32, _>(c).map(Field::Void)
                })?),
                14 => FieldTmp::Struct(value as usize),
                15 => {
                    let indices = self.list_indices.remove(&(value as usize)).ok_or_else(|| {
                        format!("couldn't find list indices at {value} in field {idx}")
                    })?;

                    FieldTmp::List(indices)
                }
                16 => Simple(read_data(tp, field_data, value, idx, |c| {
                    take::<[f32; 4]>(c)
                        .map(|[w, x, y, z]| Field::Orientation(Orientation { w, x, y, z }))
                })?),
                17 => Simple(read_data(tp, field_data, value, idx, |c| {
                    take::<[f32; 3]>(c).map(|[x, y, z]| Field::Vector(Vector { x, y, z }))
                })?),
                t => return Err(format!("Invalid field type {t} in field {idx}: {label}")),
            };

            self.fields.push(FieldReadTmp { value, label });
        }
        Ok(())
    }

    fn read_structs(&mut self) -> ESResult {
        let offset = self.h.struct_offset;
        seek_to!(self.c, offset)?;
        self.structs = Vec::with_capacity(self.h.struct_count);

        for i in 0..self.h.struct_count {
            let [tp, data, field_count] = take::<[u32; STRUCT_SIZE]>(self.c)
                .ok_or_else(|| format!("couldn't read struct {i}, starting offset {offset}"))?;
            let data = data as usize;

            let field_indices = match field_count {
                0 => vec![],
                1 => vec![data],
                _ => {
                    let start = data / DWORD_SIZE;
                    self.field_indices
                        .get(start..start + field_count as usize)
                        .ok_or_else(|| format!("couldn't read struct's {i} field indices"))?
                        .to_usize_vec()
                }
            };

            self.structs.push(StructReadTmp { tp, field_indices });
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
            tp: s.tp,
            fields: struct_fields,
        }
    }
}

fn read_data(
    tp: u32,
    field_data: &[u8],
    offset: u32,
    field_idx: usize,
    get_field: impl FnOnce(&mut Cursor<&[u8]>) -> Option<Field>,
) -> SResult<Field> {
    let data = field_data
        .get(offset as usize..)
        .ok_or_else(|| format!("invalid field data offset in field {field_idx}"))?;

    get_field(&mut Cursor::new(data)).ok_or_else(|| {
        format!(
            "couldn't read Field::{} in field {field_idx}",
            Field::repr_to_string(tp as u8)
        )
    })
}

pub fn read(bytes: &[u8]) -> SResult<Gff> {
    let mut c = Cursor::new(bytes);
    Reader::new(&mut c)
        .and_then(Reader::read)
        .map_err(|err| format!("Gff::read| {err}"))
}
