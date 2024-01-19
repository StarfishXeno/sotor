use super::{FileHead, LocString};
use sotor_macros::{EnumToInt, EnumToString, UnwrapVariant};
use std::collections::HashMap;

mod read;
mod write;

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
#[derive(Debug, Clone, PartialEq)]
pub struct Orientation {
    w: f32,
    x: f32,
    y: f32,
    z: f32,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Vector {
    x: f32,
    y: f32,
    z: f32,
}

#[repr(u8)]
#[derive(EnumToInt, EnumToString, UnwrapVariant, Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
pub struct Struct {
    pub tp: u32,
    pub fields: HashMap<String, Field>,
}
impl Struct {
    pub fn new(fields: Vec<(&str, Field)>) -> Self {
        Self::with_type(0, fields)
    }

    pub fn with_type(tp: u32, fields: Vec<(&str, Field)>) -> Self {
        Self {
            tp,
            fields: fields
                .into_iter()
                .map(|(name, value)| (name.to_owned(), value))
                .collect(),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Gff {
    pub file_head: FileHead,
    pub content: Struct,
}

#[cfg(test)]
mod tests {
    use crate::formats::{
        gff::{write, Field, Gff, LocString, Struct},
        ReadResourceNoArg,
    };

    #[test]
    fn field_value_type() {
        assert_eq!(Field::Word(0).to_int(), 2);
        assert_eq!(Field::Double(0.).to_int(), 9);
        assert_eq!(Field::Void(vec![]).to_int(), 13);
    }

    #[test]
    fn read_write() {
        let gff = Gff {
            file_head: ("TST ", "V0.0").into(),
            content: Struct::new(vec![
                ("Byte", Field::Byte(u8::MAX)),
                ("Char", Field::Char(i8::MIN)),
                ("Word", Field::Word(u16::MAX)),
                ("Short", Field::Short(i16::MIN)),
                ("Dword", Field::Dword(u32::MAX)),
                ("Int", Field::Int(i32::MIN)),
                ("Dword64", Field::Dword64(u64::MAX)),
                ("Int64", Field::Int64(i64::MIN)),
                ("Float", Field::Float(f32::MAX)),
                ("Double", Field::Double(f64::MIN)),
                ("CExoString", Field::String("CExoString".to_owned())),
                ("CResRef", Field::ResRef("CResRef".to_owned())),
                (
                    "CExoLocString",
                    Field::LocString(
                        u32::MAX,
                        vec![LocString {
                            id: 1,
                            content: "LocString".to_owned(),
                        }],
                    ),
                ),
                ("Void", Field::Void(vec![0, 1, 2, 3])),
                (
                    "Struct",
                    Field::Struct(Box::new(Struct {
                        tp: 2,
                        fields: [("NESTED".to_owned(), Field::Byte(0))].into(),
                    })),
                ),
                (
                    "List",
                    Field::List(vec![Struct {
                        tp: 0,
                        fields: [("LIST_NESTED".to_owned(), Field::Byte(0))].into(),
                    }]),
                ),
            ]),
        };
        let bytes = write(gff.clone());
        let new_gff = Gff::read(&bytes).unwrap();
        assert_eq!(gff, new_gff);
    }
}
