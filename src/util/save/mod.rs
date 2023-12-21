use super::read_file;
use crate::formats::{
    erf::{self, ERF},
    gff::{self, FieldValue, GFF},
};

mod read;

pub struct SaveGlobals {
    booleans: Vec<(String, bool)>,
    numbers: Vec<(String, u8)>,
    strings: Vec<(String, String)>,
}
pub struct SaveNfo {
    save_name: String,
    area_name: String,
    last_module: String,
    cheat_used: bool,
    time_played: u32,
}
pub struct Save {
    globals: SaveGlobals,
    nfo: SaveNfo
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
