use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub mod read;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct LocString {
    r#type: u32,
    content: String,
}

#[repr(u32)]
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
    Struct = 14,
    List = 15,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct Struct {
    r#type: u32,
    fields: HashMap<String, FieldValue>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GFF {
    file_type: String,
    file_version: String,
    structs: Vec<Struct>
}
