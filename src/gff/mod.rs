use std::collections::HashMap;

use serde::{Deserialize, Serialize};

mod read;
mod write;

pub use read::read;
pub use write::write;

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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct LocString {
    id: u32,
    content: String,
}

#[repr(u8)]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum FieldValue {
    Byte(u8) = 0,
    Char(i8) = 1,
    Word(u16) = 2,
    Short(i16) = 3,
    Dword(u32) = 4,
    Int(i32) = 5,
    Dword64(u64) = 6,
    Int64(i64) = 7,
    Float(f32) = 8,
    Double(f64) = 9,
    CExoString(String) = 10,
    CResRef(String) = 11,
    CExoLocString(Vec<LocString>) = 12,
    Void(Vec<u8>) = 13,
    // boxing it cuts FieldValue size in half
    Struct(Box<Struct>) = 14,
    List(Vec<Struct>) = 15,
}

impl FieldValue {
    fn get_type(&self) -> u32 {
        let byte: u8 = unsafe { std::mem::transmute_copy(self) }; // LOL LMAO ISHYGDDT
                                                                  // There's gotta be a safe & easy way to do this, right?
                                                                  // first byte is the specified u8 tag so Word -> 2, Double -> 9, etc
        byte as u32
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Struct {
    // type
    pub tp: u32,
    pub fields: HashMap<String, FieldValue>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct GFF {
    pub file_type: String,
    pub file_version: String,
    pub content: Struct,
}

#[cfg(test)]
mod tests {
    use super::{read, write, FieldValue, LocString, Struct, GFF};

    #[test]
    fn field_value_type() {
        assert_eq!(FieldValue::Word(0).get_type(), 2);
        assert_eq!(FieldValue::Double(0.0).get_type(), 9);
        assert_eq!(FieldValue::Void(vec![]).get_type(), 13);
    }

    #[test]
    fn read_write() {
        let gff = GFF {
            file_type: "TST ".to_owned(),
            file_version: "V0.0".to_owned(),
            content: Struct {
                tp: 0,
                fields: [
                    ("Byte".to_owned(), FieldValue::Byte(u8::MAX)),
                    ("Char".to_owned(), FieldValue::Char(i8::MIN)),
                    ("Word".to_owned(), FieldValue::Word(u16::MAX)),
                    ("Short".to_owned(), FieldValue::Short(i16::MIN)),
                    ("Dword".to_owned(), FieldValue::Dword(u32::MAX)),
                    ("Int".to_owned(), FieldValue::Int(i32::MIN)),
                    ("Dword64".to_owned(), FieldValue::Dword64(u64::MAX)),
                    ("Int64".to_owned(), FieldValue::Int64(i64::MIN)),
                    ("Float".to_owned(), FieldValue::Float(f32::MAX)),
                    ("Double".to_owned(), FieldValue::Double(f64::MIN)),
                    (
                        "CExoString".to_owned(),
                        FieldValue::CExoString("CExoString".to_owned()),
                    ),
                    (
                        "CResRef".to_owned(),
                        FieldValue::CResRef("CResRef".to_owned()),
                    ),
                    (
                        "CExoLocString".to_owned(),
                        FieldValue::CExoLocString(vec![LocString {
                            id: 1,
                            content: "LocString".to_owned(),
                        }]),
                    ),
                    ("Void".to_owned(), FieldValue::Void(vec![0, 1, 2, 3])),
                    (
                        "Struct".to_owned(),
                        FieldValue::Struct(Box::new(Struct {
                            tp: 2,
                            fields: [("NESTED".to_owned(), FieldValue::Byte(0))].into(),
                        })),
                    ),
                    (
                        "List".to_owned(),
                        FieldValue::List(vec![Struct {
                            tp: 0,
                            fields: [("LIST_NESTED".to_owned(), FieldValue::Byte(0))].into(),
                        }]),
                    ),
                ]
                .into(),
            },
        };
        let bytes = write(gff.clone()).unwrap();
        let new_gff = read(&bytes).unwrap();
        assert_eq!(gff, new_gff);
    }
}
