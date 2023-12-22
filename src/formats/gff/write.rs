use std::{
    collections::HashMap,
    io::{Cursor, Seek, Write},
};

use super::{FieldValue, FieldValueTmp, Gff, Struct, FIELD_SIZE, HEADER_SIZE};
use crate::util::{array_to_bytes, bytes_to_sized_bytes, nullpad_string, num_to_dword, DWORD_SIZE};

const MAX_LABEL_LEN: usize = 16;

struct FieldWriteTmp {
    tp: u32,
    label_index: u32,
    content: u32,
}
struct StructWriteTmp {
    tp: u32,
    my_idx: usize,
    field_count: u32,
}
pub struct Writer {
    file_type: String,
    file_version: String,
    // indices into labels
    label_map: HashMap<String, usize>,
    labels: Vec<String>,
    structs: Vec<StructWriteTmp>,
    // blocks with indices matching struct's my_idx
    fields: Vec<Vec<FieldWriteTmp>>,
    field_count: usize,
    raw_data: Vec<u8>,
    list_indices: Vec<u32>,
}

macro_rules! wf {
    ($($t:tt)*) => {{
        format!("GFF::write| {}", format!($($t)*))
    }};
}
macro_rules! wp {
    ($($t:tt)*) => {{
        println!("{}", wf!($($t)*))
    }};
}

impl Writer {
    fn new(gff: Gff) -> Self {
        let mut w = Self {
            file_type: gff.file_type,
            file_version: gff.file_version,
            label_map: HashMap::new(),
            labels: vec![],
            structs: vec![],
            fields: vec![],
            field_count: 0,
            raw_data: vec![],
            list_indices: vec![],
        };
        w.collect(gff.content);

        w
    }

    fn save_label(&mut self, label: String) -> usize {
        // Labels in GFFs must be unique
        if let Some(idx) = self.label_map.get(&label) {
            return *idx;
        }
        // all labels are exactly 16 bytes, padded with \0 if shorter
        let len = label.len();
        debug_assert!(
            len <= MAX_LABEL_LEN,
            "{}",
            wf!("labels should be 16 bytes or less")
        );
        let padded = nullpad_string(label.clone(), MAX_LABEL_LEN);

        let idx = self.labels.len();
        self.label_map.insert(label, idx);
        self.labels.push(padded);

        idx
    }

    fn save_bytes(&mut self, bytes: &[u8]) -> u32 {
        let idx = self.raw_data.len() as u32;
        self.raw_data.extend(bytes);

        idx
    }

    fn save_list_indices(&mut self, indices: &[usize]) -> u32 {
        let idx = self.list_indices.len();
        let size = indices.len() as u32;

        self.list_indices.push(size);
        self.list_indices
            .extend(indices.iter().map(|i| (*i as u32)));

        (idx * DWORD_SIZE) as u32
    }

    fn save_field(&mut self, block_idx: usize, field: FieldValueTmp, label_index: usize) {
        use FieldValue::*;

        let (tp, value): (u32, u32) = match field {
            FieldValueTmp::Simple(v) => {
                let tp = v.to_int();

                let value = match v {
                    Byte(v) => num_to_dword(v),
                    Char(v) => num_to_dword(v),
                    Word(v) => num_to_dword(v),
                    Short(v) => num_to_dword(v),
                    Dword(v) => num_to_dword(v),
                    Int(v) => num_to_dword(v),
                    Dword64(v) => self.save_bytes(&v.to_le_bytes()),
                    Int64(v) => self.save_bytes(&v.to_le_bytes()),
                    Float(v) => num_to_dword(v),
                    Double(v) => self.save_bytes(&v.to_le_bytes()),
                    String(v) => self.save_bytes(&bytes_to_sized_bytes::<DWORD_SIZE>(v.as_bytes())),
                    ResRef(v) => self.save_bytes(&bytes_to_sized_bytes::<1>(v.as_bytes())),
                    LocString(str_ref, v) => {
                        let string_count = v.len();
                        // Total Size will be added in the end
                        let mut bytes = Vec::with_capacity(string_count * 10 + 2 * DWORD_SIZE);
                        // StringRef
                        bytes.extend(str_ref.to_le_bytes());
                        // StringCount
                        bytes.extend((string_count as u32).to_le_bytes());
                        for s in v {
                            // StringID
                            bytes.extend(s.id.to_le_bytes());
                            // Length + String itself
                            bytes.extend(bytes_to_sized_bytes::<DWORD_SIZE>(s.content.as_bytes()));
                        }

                        self.save_bytes(&bytes_to_sized_bytes::<DWORD_SIZE>(&bytes))
                    }
                    Void(v) => self.save_bytes(&bytes_to_sized_bytes::<DWORD_SIZE>(&v)),
                    Orientation(v) => self.save_bytes(&array_to_bytes(&[v.w, v.x, v.y, v.z])),
                    Vector(v) => self.save_bytes(&array_to_bytes(&[v.x, v.y, v.z])),
                    _ => unreachable!(),
                };

                (tp as u32, value)
            }
            FieldValueTmp::Struct(idx) => (FieldValue::str_to_int("Struct") as u32, idx as u32),
            FieldValueTmp::List(indices) => (
                FieldValue::str_to_int("List") as u32,
                self.save_list_indices(&indices),
            ),
        };

        self.field_count += 1;
        self.fields[block_idx].push(FieldWriteTmp {
            tp,
            label_index: label_index as u32,
            content: value,
        });
    }

    fn save_struct(&mut self, tp: u32, my_idx: usize, field_count: u32) -> usize {
        let idx = self.structs.len();
        self.structs.push(StructWriteTmp {
            tp,
            my_idx,
            field_count,
        });

        idx
    }

    fn collect(&mut self, s: Struct) -> usize {
        let field_count = s.fields.len();

        let my_idx = self.fields.len();
        self.fields.push(Vec::with_capacity(field_count));

        for (label, value) in s.fields {
            let label_idx = self.save_label(label);

            match value {
                FieldValue::Struct(s) => {
                    let struct_idx = self.collect(*s);
                    self.save_field(my_idx, FieldValueTmp::Struct(struct_idx), label_idx);
                }
                FieldValue::List(list) => {
                    let mut indices = Vec::with_capacity(list.len());
                    for s in list {
                        indices.push(self.collect(s));
                    }
                    self.save_field(my_idx, FieldValueTmp::List(indices), label_idx);
                }
                v => {
                    self.save_field(my_idx, FieldValueTmp::Simple(v), label_idx);
                }
            }
        }

        self.save_struct(s.tp, my_idx, field_count as u32);

        my_idx
    }

    fn into_bytes(mut self) -> Vec<u8> {
        let buf = Vec::with_capacity(self.fields.len() * 12 * DWORD_SIZE);
        let mut cursor = Cursor::new(buf);
        let field_count = self.field_count;
        let file_type = self.file_type;
        let file_version = self.file_version;

        // sort structs propertly
        self.structs.sort_by_key(|s| s.my_idx);
        // need to flatten fields and build indices first
        let mut field_bytes = Vec::with_capacity(field_count * FIELD_SIZE * DWORD_SIZE);
        // matches self.fields indices and stores the index of the first field in a group
        let mut field_group_offsets = Vec::with_capacity(self.fields.len());
        let mut field_idx = 0;
        for group in self.fields {
            field_group_offsets.push(field_idx);
            for field in group {
                field_idx += 1;
                let data = [field.tp, field.label_index, field.content];
                field_bytes.write_all(&array_to_bytes(&data)).unwrap();
            }
        }
        // header to be filled later
        cursor.write_all(&[0; HEADER_SIZE * DWORD_SIZE]).unwrap();
        // STRUCTS
        let struct_offset = cursor.position();
        let struct_count = self.structs.len();
        let mut field_indices = Vec::with_capacity(field_count);
        for s in self.structs {
            // structs with 1 field refer to it directly instead of storing it's index in field indices
            let starting_field_idx = field_group_offsets[s.my_idx];
            let idx = if s.field_count == 1 {
                starting_field_idx
            } else {
                let index_offset = field_indices.len();
                for i in 0..s.field_count {
                    field_indices.push(starting_field_idx + i);
                }
                (index_offset * DWORD_SIZE) as u32
            };
            let data = [s.tp, idx, s.field_count];
            cursor.write_all(&array_to_bytes(&data)).unwrap();
        }
        // FIELDS
        let field_offset = cursor.position();
        cursor.write_all(&field_bytes).unwrap();
        // LABELS
        let label_offset = cursor.position();
        let label_count = self.labels.len();
        for label in self.labels {
            cursor.write_all(&label.into_bytes()).unwrap();
        }
        // FIELD DATA
        let field_data_offset = cursor.position();
        let field_data_bytes = self.raw_data.len();
        cursor.write_all(&self.raw_data).unwrap();
        // FIELD INDICES
        let field_indices_offset = cursor.position();
        let field_indices_bytes = field_indices.len() * DWORD_SIZE;
        cursor.write_all(&array_to_bytes(&field_indices)).unwrap();
        // LIST INDICES
        let list_indices_offset = cursor.position();
        let list_indices_bytes = self.list_indices.len() * DWORD_SIZE;
        cursor
            .write_all(&array_to_bytes(&self.list_indices))
            .unwrap();
        // HEADER
        if cfg!(debug_assertions) {
            wp!("Header: {file_type} {file_version}");
            wp!("Structs: {struct_count} at {struct_offset}, Fields: {field_count} at {field_offset}, Labels: {label_count} at {label_offset}");
            wp!("Field data: {field_data_bytes}B at {field_data_offset}, Field indices: {field_indices_bytes}B at {field_indices_offset}, List indices: {list_indices_bytes}B at {list_indices_offset}");
            println!("******************");
        }
        cursor.rewind().unwrap();
        cursor.write_all(file_type.as_bytes()).unwrap();
        cursor.write_all(file_version.as_bytes()).unwrap();
        cursor
            .write_all(&array_to_bytes(&[
                struct_offset as u32,
                struct_count as u32,
                field_offset as u32,
                field_count as u32,
                label_offset as u32,
                label_count as u32,
                field_data_offset as u32,
                field_data_bytes as u32,
                field_indices_offset as u32,
                field_indices_bytes as u32,
                list_indices_offset as u32,
                list_indices_bytes as u32,
            ]))
            .unwrap();

        cursor.into_inner()
    }
}
pub fn write(gff: Gff) -> Vec<u8> {
    let writer = Writer::new(gff);
    writer.into_bytes()
}
