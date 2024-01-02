use super::LocString;
use serde::{Deserialize, Serialize};
use sotor_macros::{EnumToInt, UnwrapVariant};
use std::collections::HashMap;

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
enum FieldTmp {
    // simple value
    Simple(Field),
    // index into struct array
    Struct(usize),
    // list of indices into struct array
    List(Vec<usize>),
}
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Orientation {
    w: f32,
    x: f32,
    y: f32,
    z: f32,
}
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Vector {
    x: f32,
    y: f32,
    z: f32,
}

#[repr(u8)]
#[derive(EnumToInt, UnwrapVariant, Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Field {
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
    String(String) = 10,
    ResRef(String) = 11,
    LocString(u32, Vec<LocString>) = 12,
    Void(Vec<u8>) = 13,
    // boxing it cuts FieldValue size in half
    Struct(Box<Struct>) = 14,
    List(Vec<Struct>) = 15,
    Orientation(Orientation) = 16,
    Vector(Vector) = 17,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Struct {
    // type
    pub tp: u32,
    pub fields: HashMap<String, Field>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Gff {
    pub file_type: String,
    pub file_version: String,
    pub content: Struct,
}

#[cfg(test)]
mod tests {
    use super::{read, write, Field, Gff, LocString, Struct};

    #[test]
    fn field_value_type() {
        assert_eq!(Field::Word(0).to_int(), 2);
        assert_eq!(Field::Double(0.0).to_int(), 9);
        assert_eq!(Field::Void(vec![]).to_int(), 13);
    }

    #[test]
    fn read_write() {
        let gff = Gff {
            file_type: "TST ".to_owned(),
            file_version: "V0.0".to_owned(),
            content: Struct {
                tp: 0,
                fields: [
                    ("Byte".to_owned(), Field::Byte(u8::MAX)),
                    ("Char".to_owned(), Field::Char(i8::MIN)),
                    ("Word".to_owned(), Field::Word(u16::MAX)),
                    ("Short".to_owned(), Field::Short(i16::MIN)),
                    ("Dword".to_owned(), Field::Dword(u32::MAX)),
                    ("Int".to_owned(), Field::Int(i32::MIN)),
                    ("Dword64".to_owned(), Field::Dword64(u64::MAX)),
                    ("Int64".to_owned(), Field::Int64(i64::MIN)),
                    ("Float".to_owned(), Field::Float(f32::MAX)),
                    ("Double".to_owned(), Field::Double(f64::MIN)),
                    (
                        "CExoString".to_owned(),
                        Field::String("CExoString".to_owned()),
                    ),
                    ("CResRef".to_owned(), Field::ResRef("CResRef".to_owned())),
                    (
                        "CExoLocString".to_owned(),
                        Field::LocString(
                            u32::MAX,
                            vec![LocString {
                                id: 1,
                                content: "LocString".to_owned(),
                            }],
                        ),
                    ),
                    ("Void".to_owned(), Field::Void(vec![0, 1, 2, 3])),
                    (
                        "Struct".to_owned(),
                        Field::Struct(Box::new(Struct {
                            tp: 2,
                            fields: [("NESTED".to_owned(), Field::Byte(0))].into(),
                        })),
                    ),
                    (
                        "List".to_owned(),
                        Field::List(vec![Struct {
                            tp: 0,
                            fields: [("LIST_NESTED".to_owned(), Field::Byte(0))].into(),
                        }]),
                    ),
                ]
                .into(),
            },
        };
        let bytes = write(gff.clone());
        let new_gff = read(&bytes).unwrap();
        assert_eq!(gff, new_gff);
    }
}
