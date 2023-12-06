use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub mod read;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LocString {
    id: u32,
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
    Struct(Struct) = 14,
    List(Vec<Struct>) = 15,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Struct {
    pub r#type: u32,
    pub fields: HashMap<String, FieldValue>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GFF {
    pub file_type: String,
    pub file_version: String,
    pub content: Struct
}
