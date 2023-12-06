use crate::{
    gff::{FieldValue, LocString, Struct},
    util::{
        bytes_to_exo_string, bytes_to_string, cast_bytes, read_bytes, read_chunks, read_dwords,
        ToUSizeVec, DWORD_SIZE,
    },
};

use super::GFF;
use std::{
    collections::HashMap,
    fs,
    io::BufReader,
    io::{prelude::*, SeekFrom},
    str::{self},
};

// 7 pairs of DWORDS
const HEADER_SIZE: usize = 7 * 2;
// 3 DWORDS
const STRUCT_SIZE: usize = 3;
// 3 DWORDS
const FIELD_SIZE: usize = 3;

#[derive(Debug)]
enum FieldValueTmp {
    // simple value
    Simple(FieldValue),
    // index into struct array
    Struct(usize),
    // list of indices into struct array
    List(Vec<usize>),
}

#[derive(Debug)]
struct FieldTmp {
    value: FieldValueTmp,
    label: String,
}

#[derive(Debug)]
struct StructTmp {
    r#type: u32,
    field_indices: Vec<usize>,
}

fn unwrap_deferred_field(
    f: &FieldValueTmp,
    fields: &Vec<FieldTmp>,
    structs: &Vec<StructTmp>,
) -> FieldValue {
    match f {
        FieldValueTmp::Simple(value) => value.clone(),
        FieldValueTmp::Struct(idx) => {
            FieldValue::Struct(transform_struct(&structs[*idx], fields, structs))
        }
        FieldValueTmp::List(indices) => {
            let structs: Vec<Struct> = indices
                .into_iter()
                .map(|i| transform_struct(&structs[*i], fields, structs))
                .collect();

            FieldValue::List(structs)
        }
    }
}

fn transform_struct(s: &StructTmp, fields: &Vec<FieldTmp>, structs: &Vec<StructTmp>) -> Struct {
    let mut struct_fields = HashMap::with_capacity(s.field_indices.len());
    for idx in s.field_indices.iter() {
        let field = &fields[*idx];
        struct_fields.insert(
            field.label.clone(),
            unwrap_deferred_field(&field.value, fields, structs),
        );
    }
    Struct {
        r#type: s.r#type,
        fields: struct_fields,
    }
}

macro_rules! seek {
    ($reader:ident, $offset:ident) => {
        $reader
            .seek(SeekFrom::Start($offset as u64))
            .map_err(|_| format!("Invalid {}", stringify!($offset)))?;
    };
}

pub fn read(path: &str) -> Result<GFF, String> {
    let file = fs::File::open(path).map_err(|_| "Couldn't read file")?;
    let mut reader = BufReader::new(file);

    // HEADER
    let dwords = read_dwords(&mut reader, HEADER_SIZE).map_err(|_| "Couldn't read header")?;
    let file_type =
        bytes_to_string(&dwords[0].to_ne_bytes()).map_err(|_| "Couldn't parse file_type")?;
    let file_version =
        bytes_to_string(&dwords[1].to_ne_bytes()).map_err(|_| "Couldn't parse file_version")?;
    let dwords = dwords[2..].to_usize_vec();
    let struct_offset = dwords[0];
    let struct_count = dwords[1];
    let field_offset = dwords[2];
    let field_count = dwords[3];
    let label_offset = dwords[4];
    let label_count = dwords[5];
    let field_data_offset = dwords[6];
    let field_data_bytes = dwords[7];
    let field_indices_offset = dwords[8];
    let field_indices_bytes = dwords[9];
    let list_indices_offset = dwords[10];
    let list_indices_bytes = dwords[11];

    println!("Header: {file_type} {file_version}");
    println!("Structs: {struct_count} at {struct_offset}, Fields: {field_count} at {field_offset}, Labels: {label_count} at {label_offset}");
    println!("Field data: {field_data_bytes} at {field_data_offset}, Field indices: {field_indices_bytes} at {field_indices_offset}, List indices: {list_indices_bytes} at {list_indices_offset}");
    println!("******************");

    // LIST INDICES
    let mut list_indices = HashMap::new();
    seek!(reader, list_indices_offset);

    loop {
        let offset = reader
            .stream_position()
            .map_err(|_| "couldn't get reader offset")? as usize;
        let relative_offset = offset - list_indices_offset;
        if relative_offset == list_indices_bytes {
            break;
        }
        let dwords = read_dwords(&mut reader, 1).map_err(|_| {
            format!(
                "Couldn't read list indices size {relative_offset}, starting offset {}",
                list_indices_offset
            )
        })?;
        let size = dwords[0] as usize;
        let dwords = read_dwords(&mut reader, size).map_err(|_| {
            format!(
                "Couldn't read list indices at {relative_offset}, starting offset {}",
                list_indices_offset
            )
        })?;
        list_indices.insert(relative_offset, dwords.to_usize_vec());
    }
    println!("List indices: {}", list_indices.len());
    println!("******************");

    // FIELD INDICES
    seek!(reader, field_indices_offset);

    let field_indices: Vec<usize> = read_dwords(&mut reader, field_indices_bytes / DWORD_SIZE)
        .map_err(|_| {
            format!(
                "Couldn't read field indices, starting offset {}",
                field_indices_offset
            )
        })?
        .to_usize_vec();
    println!("Field indices: {}", field_indices.len());
    println!("******************");

    // FIELD DATA
    seek!(reader, field_data_offset);

    let field_data = read_bytes(&mut reader, field_data_bytes).map_err(|_| {
        format!(
            "Couldn't read field data, starting offset {}",
            field_data_offset
        )
    })?;
    println!("Field data: {}", field_data.len());
    println!("******************");

    // LABELS
    seek!(reader, label_offset);

    let mut labels: Vec<String> = Vec::with_capacity(label_count);
    let chunks = read_chunks(&mut reader, label_count, 16).map_err(|_| {
        format!(
            "Couldn't read field indices, starting offset {}",
            field_indices_offset
        )
    })?;
    for c in chunks {
        let label = str::from_utf8(&c)
            .unwrap_or("INVALID LABEL")
            .trim_matches('\0')
            .to_owned();
        labels.push(label);
    }

    println!("Labels: {}", labels.len());
    println!("******************");

    // STRUCTS
    let mut structs = Vec::with_capacity(struct_count);
    seek!(reader, struct_offset);

    for i in 0..struct_count {
        let dwords = read_dwords(&mut reader, STRUCT_SIZE).map_err(|_| {
            format!(
                "Couldn't read struct {i}, starting offset {}",
                struct_offset
            )
        })?;

        let data_or_data_offset = dwords[1] as usize;
        let field_count = dwords[2] as usize;
        let struct_field_indices = if field_count == 1 {
            vec![data_or_data_offset]
        } else {
            let start = data_or_data_offset / DWORD_SIZE;
            field_indices[start..start + field_count].into()
        };

        structs.push(StructTmp {
            r#type: dwords[0],
            field_indices: struct_field_indices,
        });
    }
    println!("Structs: {}", structs.len());
    println!("******************");

    // FIELDS
    let mut fields = Vec::with_capacity(field_count);
    seek!(reader, field_offset);

    for i in 0..field_count {
        use FieldValueTmp::*;
        let dwords = read_dwords(&mut reader, FIELD_SIZE)
            .map_err(|_| format!("Couldn't read field {i}, starting offset {}", field_offset))?
            .to_usize_vec();
        let label = labels[dwords[1]].clone();
        let inner = dwords[2];

        let value = match dwords[0] {
            0 => Simple(FieldValue::Byte(inner as u8)),
            1 => Simple(FieldValue::Char(inner as i8)),
            2 => Simple(FieldValue::Word(inner as u16)),
            3 => Simple(FieldValue::Short(inner as i16)),
            4 => Simple(FieldValue::Dword(inner as u32)),
            5 => Simple(FieldValue::Int(inner as i32)),
            6 => Simple(FieldValue::Dword64(cast_bytes!(&field_data[inner..], u64))),
            7 => Simple(FieldValue::Int64(cast_bytes!(&field_data[inner..], i64))),
            8 => Simple(FieldValue::Float(inner as f32)),
            9 => Simple(FieldValue::Double(cast_bytes!(&field_data[inner..], f64))),
            10 => Simple(FieldValue::CExoString(
                bytes_to_exo_string!(&field_data[inner..], u32)
                    .map_err(|_| format!("Invalid CExoString data in field {i}: {label}"))?,
            )),
            11 => Simple(FieldValue::CResRef(
                bytes_to_exo_string!(&field_data[inner..], u8)
                    .map_err(|_| format!("Invalid CResRef data in field {i}: {label}"))?,
            )),
            12 => {
                let count = cast_bytes!(&field_data[inner + DWORD_SIZE * 2..], u32);
                let mut offset = inner + DWORD_SIZE * 3;
                let mut strings = Vec::with_capacity(count as usize);
                for j in 0..count {
                    let id = cast_bytes!(&field_data[offset..], u32);
                    offset += DWORD_SIZE;
                    let length = cast_bytes!(&field_data[offset..], u32) as usize;
                    offset += DWORD_SIZE;
                    let content =
                        bytes_to_string(&field_data[offset..offset + length]).map_err(|_| {
                            format!("Invalid CExoLocalString {j} in field {i}: {label}")
                        })?;
                    strings.push(LocString { id, content });
                    offset += length;
                }
                Simple(FieldValue::CExoLocString(strings))
            }
            13 => Simple(FieldValue::Void(
                crate::util::bytes_to_sized_bytes!(&field_data[inner..], u32).into(),
            )),
            14 => Struct(inner),
            15 => {
                let indices = list_indices
                    .remove(&inner)
                    .ok_or(format!("Couldn't find list indices {i}"))?;
                List(indices)
            }
            t => return Err(format!("Invalid field type {t} in field {i}: {label}")),
        };

        fields.push(FieldTmp { value, label });
    }

    println!("Fields: {}", fields.len());
    println!("******************");

    Ok(GFF {
        file_type,
        file_version,
        content: transform_struct(&structs[0], &fields, &structs),
    })
}
