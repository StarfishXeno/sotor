use std::collections::HashMap;

use super::{FieldValue, FieldValueTmp, Struct, GFF};

const MAX_LABEL_LEN: usize = 16;

struct FieldWriteTmp {
    tp: u32,
    label_index: u32,
    content: u32,
}
struct StructWriteTmp {
    tp: u32,
    index: usize,
    field_count: u32,
}
pub struct Writer {
    // indices into labels
    label_map: HashMap<String, usize>,
    labels: Vec<String>,
    structs: Vec<StructWriteTmp>,
    fields: Vec<Vec<FieldWriteTmp>>,
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

    fn save_field(&mut self, block_idx: usize, field: FieldValueTmp, label_idx: usize) {}

    fn save_struct(&mut self, tp: u32, index: usize, field_count: u32) -> usize {
        let idx = self.structs.len();
        self.structs.push(StructWriteTmp {
            tp,
            index,
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
                    let struct_idx = self.collect(s);
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
        };
        writer.collect(gff.content);
    }
}
