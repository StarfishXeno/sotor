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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LocString {
    id: u32,
    content: String,
}

#[repr(u8)]
#[derive(Debug, Serialize, Deserialize, Clone)]
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


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Struct {
    // type
    pub tp: u32,
    pub fields: HashMap<String, FieldValue>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GFF {
    pub file_type: String,
    pub file_version: String,
    pub content: Struct,
}

#[cfg(test)]
mod tests {
    use crate::gff::FieldValue;

    #[test]
    fn field_value_type() {
        assert_eq!(FieldValue::Word(0).get_type(), 2);
        assert_eq!(FieldValue::Double(0.0).get_type(), 9);
        assert_eq!(FieldValue::Void(vec![]).get_type(), 13);
    }

}