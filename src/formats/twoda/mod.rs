use ahash::HashMap;
use sotor_macros::UnwrapVariant;

mod read;
pub use read::*;

#[derive(Debug, Clone, Copy)]
pub enum TwoDAType {
    String,
    Int,
}

#[derive(UnwrapVariant, Debug, Clone)]
pub enum TwoDAValue {
    String(String),
    Int(i32),
}
#[derive(Debug, Clone)]
pub struct TwoDA(pub Vec<HashMap<String, Option<TwoDAValue>>>);
