use crate::{
    formats::{
        erf::{self, Erf},
        gff::{self, Gff},
    },
    util::{read_file, write_file},
};

use self::write::Writer;

mod read;
mod write;

#[derive(Debug, Clone, PartialEq)]
pub struct PartyMember {
    pub idx: usize,
    pub leader: bool,
}
#[derive(Debug, Clone, PartialEq)]
pub struct AvailablePartyMember {
    pub available: bool,
    pub selectable: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct JournalEntry {
    pub id: String,
    pub state: i32,
    pub date: u32,
    pub time: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PartyTable {
    pub journal: Vec<JournalEntry>,
    pub cheats_used: bool,
    pub credits: u32,
    pub members: Vec<PartyMember>,
    pub available_members: Vec<AvailablePartyMember>,
    pub party_xp: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Global<T> {
    pub name: String,
    pub value: T,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Globals {
    pub booleans: Vec<Global<bool>>,
    pub numbers: Vec<Global<u8>>,
    pub strings: Vec<Global<String>>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Nfo {
    pub save_name: String,
    pub area_name: String,
    pub last_module: String,
    pub cheats_used: bool,
    pub time_played: u32,
}
#[derive(Debug, Clone, PartialEq)]
pub enum Game {
    One,
    Two,
}
impl Game {
    pub fn get_party_names(&self) -> &'static [&'static str] {
        static PARTY_1: &[&str] = &[
            "Bastila",
            "Canderous",
            "Carth",
            "HK-47",
            "Jolee",
            "Juhani",
            "Mission",
            "T3-M4",
            "Zaalbar",
        ];

        static PARTY_2: &[&str] = &[
            "Atton",
            "Bao-Dur",
            "Mandalore",
            "G0-T0",
            "Handmaiden",
            "HK-47",
            "Kreia",
            "Mira",
            "T3-M4",
            "Visas",
            "Hanharr",
            "Disciple",
        ];

        match self {
            Game::One => PARTY_1,
            Game::Two => PARTY_2,
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
struct SaveInternals {
    nfo: Gff,
    globals: Gff,
    party_table: Gff,
    erf: Erf,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Save {
    pub game: Game,
    pub globals: Globals,
    pub nfo: Nfo,
    pub party_table: PartyTable,

    inner: SaveInternals,
}

const GFF_NAMES: &[&str] = &["savenfo.res", "globalvars.res", "partytable.res"];
const ERF_NAME: &str = "savegame.sav";

macro_rules! sf {
    ($($t:tt)*) => {{
        format!("Save| {}", format!($($t)*))
    }};
}

impl Save {
    pub fn read_from_directory(path: &str, game: Game) -> Result<Self, String> {
        let mut gffs = Vec::with_capacity(GFF_NAMES.len());

        for name in GFF_NAMES {
            let file = read_file(&(path.to_owned() + name))
                .map_err(|_| sf!("Couldn't read GFF file {name}"))?;
            let gff = gff::read(&file)?;
            gffs.push(gff);
        }

        let erf_bytes =
            read_file(&(path.to_owned() + ERF_NAME)).map_err(|_| sf!("Couldn't read ERF file"))?;
        let erf = erf::read(&erf_bytes)?;
        let reader = read::Reader::new(
            SaveInternals {
                nfo: gffs.remove(0),
                globals: gffs.remove(0),
                party_table: gffs.remove(0),
                erf,
            },
            game,
        );

        reader.process()
    }
    pub fn save_to_directory(path: &str, save: Save) -> Result<Save, String> {
        let new_save = Writer::new(save).process();
        for (name, gff) in GFF_NAMES.iter().zip([
            &new_save.inner.nfo,
            &new_save.inner.globals,
            &new_save.inner.party_table,
        ]) {
            let bytes = gff::write(gff.clone());
            write_file(&(path.to_owned() + name), &bytes)
                .map_err(|err| sf!("Couldn't write gff file {name}: {}", err.to_string()))?;
        }

        Ok(new_save)
    }
}
