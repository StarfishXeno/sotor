use std::{collections::HashMap};


use crate::util::{ToBytes, DWORD_SIZE, num_to_word, bytes_to_sized_bytes};

use super::{FieldValue, FieldValueTmp, Struct, GFF};

const MAX_LABEL_LEN: usize = 16;

struct FieldWriteTmp {
    tp: u32,
    label_index: u32,
    content: u32,
}
struct StructWriteTmp {
    tp: u32,
    field_index: usize,
    field_count: u32,
}
pub struct Writer {
    // indices into labels
    label_map: HashMap<String, usize>,
    labels: Vec<String>,
    structs: Vec<StructWriteTmp>,
    fields: Vec<Vec<FieldWriteTmp>>,
    raw_data: Vec<u8>,
}



impl Writer {
    fn save_label(&mut self, label: String) -> usize {
        // Labels in GFFs must be unique
        if let Some(idx) = self.label_map.get(&label) {
            return *idx;
        }
        let len = label.len();
        // all labels are exactly 16 bytes, padded with \0 if shorter
        debug_assert!(
            len <= MAX_LABEL_LEN,
            "GFF::Writer| labels should be 16 bytes or less"
        );
        let mut label_padded = label.clone();
        if len < MAX_LABEL_LEN {
            label_padded.push_str(&"\0".repeat(MAX_LABEL_LEN - len))
        }
        debug_assert!(label_padded.len() == 16);
        // saving
        let idx = self.labels.len();
        self.label_map.insert(label, idx);
        self.labels.push(label_padded);

        idx
    }

    fn save_bytes(&mut self, bytes: &[u8]) -> u32 {
        let index = self.raw_data.len() as u32;
        self.raw_data.extend(bytes);

        index
    }

    fn save_field(&mut self, block_idx: usize, field: FieldValueTmp, label_index: usize) {
        use FieldValue::*;

        let (tp, value): (u32, u32) = match field {
            FieldValueTmp::Simple(v) => {
                let tp = v.get_type();

                let value = match v {
                    Byte(v) => num_to_word(v),
                    Char(v) => num_to_word(v),
                    Word(v) => num_to_word(v),
                    Short(v) => num_to_word(v),
                    Dword(v) => num_to_word(v),
                    Int(v) => num_to_word(v),
                    Dword64(v) => self.save_bytes(&v.to_le_bytes()),
                    Int64(v) => self.save_bytes(&v.to_le_bytes()),
                    Float(v) => num_to_word(v),
                    Double(v) => self.save_bytes(&v.to_le_bytes()),
                    CExoString(v) => self.save_bytes(&bytes_to_sized_bytes::<DWORD_SIZE>(&v.into_bytes())),
                    CResRef(v) => self.save_bytes(&bytes_to_sized_bytes::<1>(&v.into_bytes())),
                    CExoLocString(v) => {
                        let string_count = v.len();
                        // Total Size will be added in the end
                        let mut bytes = Vec::with_capacity(string_count * 10 + 3);
                        // StringRef, 0xFFFFFFFF means empty (for now?)
                        bytes.extend(u32::MAX.to_le_bytes());
                        // StringCount
                        bytes.extend((string_count as u32).to_le_bytes());
                        for s in v {
                            // StringID
                            bytes.extend(s.id.to_be_bytes());
                            // Length + String itself
                            bytes.extend(bytes_to_sized_bytes::<DWORD_SIZE>(
                                &s.content.into_bytes(),
                            ));
                        }

                        self.save_bytes(&bytes_to_sized_bytes::<DWORD_SIZE>(&bytes))
                    }
                    Void(v) => self.save_bytes(&bytes_to_sized_bytes::<DWORD_SIZE>(&v)),
                    _ => unreachable!(),
                };
                (tp, value)
            }
            _ => (0, 0),
        };
        self.fields[block_idx].push(FieldWriteTmp {
            tp,
            label_index: label_index as u32,
            content: value,
        })
    }

    fn save_struct(&mut self, tp: u32, field_index: usize, field_count: u32) -> usize {
        let idx = self.structs.len();
        self.structs.push(StructWriteTmp {
            tp,
            field_index,
            field_count,
        });

        idx
    }

    fn collect(&mut self, s: Struct) -> usize {
        let field_count = s.fields.len();
        debug_assert!(
            field_count != 0,
            "GFF::Writer| empty structs shouldn't exist"
        );

        let my_idx = self.fields.len();
        self.fields.push(Vec::with_capacity(field_count));

        for (label, value) in s.fields.into_iter() {
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

        self.save_struct(s.tp, my_idx, field_count as u32)
    }

    pub fn to_bytes(gff: GFF) {
        let mut writer = Self {
            label_map: HashMap::new(),
            labels: vec![],
            structs: vec![],
            fields: vec![],
            raw_data: vec![],
        };
        writer.collect(gff.content);
    }
}
