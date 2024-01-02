use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{
    formats::{
        erf::{self, Erf},
        gff::{self, Field, Gff},
    },
    util::read_file,
};

mod read;

#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Global<T> {
    name: String,
    value: T,
}

#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Globals {
    booleans: Vec<Global<bool>>,
    numbers: Vec<Global<u8>>,
    strings: Vec<Global<String>>,
}
#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Nfo {
    pub save_name: String,
    pub area_name: String,
    pub last_module: String,
    pub cheats_used: bool,
    pub time_played: u32,
}
#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Save {
    pub globals: Globals,
    pub nfo: Nfo,
}

const GFF_NAMES: &[&str] = &["savenfo.res", "globalvars.res", "partytable.res"];
const ERF_NAME: &str = "savegame.sav";
const ERF_GFF_NAMES: &[&str] = &["INVENTORY"];

macro_rules! sf {
    ($($t:tt)*) => {{
        format!("Save| {}", format!($($t)*))
    }};
}

impl Save {
    pub fn read_from_directory(path: &str) -> Result<Self, String> {
        let mut gffs = Vec::with_capacity(GFF_NAMES.len());

        for name in GFF_NAMES {
            let file = read_file(&(path.to_owned() + "/" + name))
                .map_err(|_| sf!("Couldn't read GFF file {name}"))?;
            let gff = gff::read(&file)?;
            gffs.push(gff);
        }

        let erf_bytes = read_file(&(path.to_owned() + "/" + ERF_NAME))
            .map_err(|_| sf!("Couldn't read ERF file"))?;
        let erf = erf::read(&erf_bytes)?;
        let reader = read::SaveReader::new(&gffs[0], &gffs[1], &gffs[2], &erf);

        reader.process()
    }
}
