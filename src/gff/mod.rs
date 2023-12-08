use std::collections::HashMap;

use serde::{Deserialize, Serialize};

mod read;
mod write;

pub use write::Writer;
pub use read::read;

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
    Struct(Struct) = 14,
    List(Vec<Struct>) = 15,
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
    pub content: Struct
}
