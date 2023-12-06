use crate::{
    gff::{FieldValue, Struct},
    util::{
        bytes_to_exo_string, bytes_to_string, cast_bytes, read_bytes, read_chunks, read_dwords,
        DWORD_SIZE,
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
pub struct Field {
    value: FieldValue,
    label: String,
}

pub fn read(path: &str) -> Result<GFF, String> {
    let file = fs::File::open(path).unwrap();
    let mut reader = BufReader::new(file);

    // HEADER
    let dwords = read_dwords(&mut reader, HEADER_SIZE).map_err(|_| "Couldn't read header")?;
    let file_type =
        bytes_to_string(&dwords[0].to_ne_bytes()).map_err(|_| "Couldn't parse file_type")?;
    let file_version =
        bytes_to_string(&dwords[1].to_ne_bytes()).map_err(|_| "Couldn't parse file_version")?;
    let struct_offset = dwords[2];
    let struct_count = dwords[3];
    let field_offset = dwords[4];
    let field_count = dwords[5];
    let label_offset = dwords[6];
    let label_count = dwords[7];
    let field_data_offset = dwords[8];
    let field_data_bytes = dwords[9];
    let field_indices_offset = dwords[10];
    let field_indices_bytes = dwords[11];
    let list_indices_offset = dwords[12];
    let list_indices_bytes = dwords[13];

    println!("Header: {file_type} {file_version}");
    println!("Structs: {struct_count} at {struct_offset}, Fields: {field_count} at {field_offset}, Labels: {label_count} at {label_offset}");
    println!("Field data: {field_data_bytes} at {field_data_offset}, Field indices: {field_indices_bytes} at {field_indices_offset}, List indices: {list_indices_bytes} at {list_indices_offset}");
    println!("******************");

    // LIST INDICES
    let mut list_indices = HashMap::new();
    reader
        .seek(SeekFrom::Start(list_indices_offset as u64))
        .map_err(|_| "Invalid list_indices_offset")?;
    let mut indices_bytes_read = 0;
    while indices_bytes_read < list_indices_bytes {
        let offset = reader
            .stream_position()
            .map_err(|_| "couldn't get reader offset")? as u32;
        let relative_offset = offset - list_indices_offset;
        let dwords = read_dwords(&mut reader, 1).map_err(|_| {
            format!(
                "Couldn't read list indices size {relative_offset}, starting offset {}",
                list_indices_offset
            )
        })?;
        let size = dwords[0];
        let dwords = read_dwords(&mut reader, size as usize).map_err(|_| {
            format!(
                "Couldn't read list indices {relative_offset}, starting offset {}",
                list_indices_offset
            )
        })?;
        list_indices.insert(relative_offset, dwords);
        indices_bytes_read += relative_offset;
    }
    println!("List incdices: {}", list_indices.len());
    println!("******************");

    // FIELD INDICES
    reader
        .seek(SeekFrom::Start(field_indices_offset as u64))
        .map_err(|_| "Invalid field_indices_offset")?;

    let field_indices: Vec<usize> =
        read_dwords(&mut reader, field_indices_bytes as usize / DWORD_SIZE)
            .map_err(|_| {
                format!(
                    "Couldn't read field indices, starting offset {}",
                    field_indices_offset
                )
            })?
            .into_iter()
            .map(|i| i as usize)
            .collect();
    println!("Field indices: {}", field_indices.len());
    println!("******************");

    // FIELD DATA
    reader
        .seek(SeekFrom::Start(field_data_offset as u64))
        .map_err(|_| "Invalid field_data_offset")?;

    let field_data = read_bytes(&mut reader, field_data_bytes as usize).map_err(|_| {
        format!(
            "Couldn't read field data, starting offset {}",
            field_data_offset
        )
    })?;
    println!("Field data: {}", field_data.len());
    println!("******************");

    // LABELS
    reader
        .seek(SeekFrom::Start(label_offset as u64))
        .map_err(|_| "Invalid label_offset")?;

    let labels: Vec<String> = read_chunks(&mut reader, label_count as usize, 16)
        .map_err(|_| {
            format!(
                "Couldn't read field indices, starting offset {}",
                field_indices_offset
            )
        })?
        .into_iter()
        .map(|c| {
            str::from_utf8(&c)
                .unwrap_or("INVALID LABEL")
                .trim_matches('\0')
                .to_owned()
        })
        .collect();

    println!("Labels: {}", labels.len());
    println!("******************");

    // FIELDS
    let mut fields = Vec::with_capacity(field_count as usize);
    reader
        .seek(SeekFrom::Start(field_offset as u64))
        .map_err(|_| "Invalid field_offset")?;
    for i in 0..field_count {
        let dwords = read_dwords(&mut reader, FIELD_SIZE)
            .map_err(|_| format!("Couldn't read field {i}, starting offset {}", field_offset))?;
        let label = labels[dwords[1] as usize].clone();
        let inner = dwords[2] as usize;

        let value = match dwords[0] {
            0 => FieldValue::Byte(inner as u8),
            1 => FieldValue::Char(inner as i8),
            2 => FieldValue::Word(inner as u16),
            3 => FieldValue::Short(inner as i16),
            4 => FieldValue::Dword(inner as u32),
            5 => FieldValue::Int(inner as i32),
            6 => FieldValue::Dword64(cast_bytes!(&field_data[inner..], u64)),
            7 => FieldValue::Int64(cast_bytes!(&field_data[inner..], i64)),
            8 => FieldValue::Float(inner as f32),
            9 => FieldValue::Double(cast_bytes!(&field_data[inner..], f64)),
            10 => FieldValue::CExoString(
                bytes_to_exo_string!(&field_data[inner..], u32)
                    .map_err(|_| format!("Invalid CExoString data in field {i}: {label}"))?,
            ),
            11 => FieldValue::CResRef(
                bytes_to_exo_string!(&field_data[inner..], u8)
                    .map_err(|_| format!("Invalid CResRef data in field {i}: {label}"))?,
            ),
            //12=> FieldValue::CExoLocString(Vec<LocString>),
            13 => FieldValue::Void(
                crate::util::bytes_to_sized_bytes!(&field_data[inner..], u32).into(),
            ),
            14 => FieldValue::Struct,
            15 => FieldValue::List,
            t => return Err(format!("Invalid field type {t} in field {i}: {label}")),
        };

        fields.push(Field { value, label });
    }
    println!("Fields: {}", fields.len());
    println!("******************");

    // STRUCTS
    let mut structs = Vec::with_capacity(struct_count as usize);
    reader
        .seek(SeekFrom::Start(struct_offset as u64))
        .map_err(|_| "Invalid struct_offset")?;

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
        let mut struct_fields = HashMap::with_capacity(field_count);
        for idx in struct_field_indices {
            let field = &fields[idx];
            struct_fields.insert(field.label.clone(), field.value.clone());
        }

        structs.push(Struct {
            r#type: dwords[0],
            fields: struct_fields,
        });
    }
    println!("Structs: {}", structs.len());
    println!("******************");

    Ok(GFF {
        file_type,
        file_version,
        structs
    })
}
