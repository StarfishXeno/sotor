use std::collections::HashMap;

mod read;

pub use read::*;

#[derive(Debug, Clone, Copy)]
pub enum TwoDAType {
    String,
    Int,
}

#[derive(Debug, Clone)]
pub enum TwoDAValue {
    String(String),
    Int(i32),
}
#[derive(Debug, Clone)]
pub struct TwoDA(pub Vec<HashMap<String, Option<TwoDAValue>>>);
