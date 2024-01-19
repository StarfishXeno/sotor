use crate::util::SResult;
use serde::{Deserialize, Serialize};
use sotor_macros::{EnumFromInt, EnumToString};

pub mod bif;
pub mod erf;
pub mod gff;
pub mod key;
pub mod tlk;
pub mod twoda;

#[derive(Debug, Default, PartialEq, Clone)]
pub struct FileHead {
    pub tp: String,
    pub version: String,
}

impl From<(&str, &str)> for FileHead {
    fn from((tp, version): (&str, &str)) -> Self {
        debug_assert!(tp.len() == 4);
        debug_assert!(version.len() == 4);

        Self {
            tp: tp.to_owned(),
            version: version.to_owned(),
        }
    }
}

#[repr(u16)]
#[derive(EnumFromInt, EnumToString, Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum ResourceType {
    Unknown = 0,
    Txt = 10,
    Are = 2012,
    Ifo = 2014,
    Twoda = 2017,
    Git = 2023,
    Uti = 2025,
    Utc = 2027,
    Fac = 2038,
    Sav = 2057,
    Tpc = 3007,
}

impl ResourceType {
    pub fn to_extension(self) -> String {
        match self {
            Self::Twoda => "2da".to_owned(),
            _ => self.to_str().to_lowercase(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct ResourceKey(pub String, pub ResourceType);

impl From<(String, ResourceType)> for ResourceKey {
    fn from((str, tp): (String, ResourceType)) -> Self {
        Self(str, tp)
    }
}

impl From<(&str, ResourceType)> for ResourceKey {
    fn from((str, tp): (&str, ResourceType)) -> Self {
        Self(str.to_owned(), tp)
    }
}

pub trait ReadResource<'a, Arg: 'a + Copy>: Sized {
    fn get_type() -> ResourceType;
    fn read(bytes: &[u8], arg: Arg) -> SResult<Self>;
}

pub trait ReadResourceNoArg: Sized {
    fn get_type() -> ResourceType;
    fn read(bytes: &[u8]) -> SResult<Self>;
}

impl<'a, T: ReadResource<'a, ()>> ReadResourceNoArg for T {
    fn get_type() -> ResourceType {
        T::get_type()
    }

    fn read(bytes: &[u8]) -> SResult<Self> {
        T::read(bytes, ())
    }
}

macro_rules! impl_read_resource {
    ($t:ident, $reader:ident, $arg:ty, $type:expr) => {
        impl<'a> ReadResource<'a, $arg> for $t {
            fn get_type() -> ResourceType {
                $type
            }
            fn read(bytes: &[u8], arg: $arg) -> SResult<Self> {
                let c = &mut Cursor::new(bytes);
                $reader::new(c, arg)
                    .read()
                    .map_err(|err| format!("{}::read| {err}", stringify!($t)))
            }
        }
    };

    ($t:ident, $reader:ident, $arg:ty) => {
        impl_read_resource!($t, $reader, $arg, ResourceType::Unknown);
    };

    ($t:ident, $reader:ident) => {
        impl_read_resource!($t, $reader, ());
    };
}
pub(crate) use impl_read_resource;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct LocString {
    pub id: u32,
    pub content: String,
}
