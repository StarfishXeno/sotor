use crate::{
    formats::{erf, gff},
    util::read_file,
};

mod read;

#[derive(Debug, Clone, PartialEq)]
pub struct PartyMember {
    pub idx: usize,
    pub is_leader: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct JournalEntry {
    id: String,
    state: i32,
    date: u32,
    time: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PartyTable {
    pub journal: Vec<JournalEntry>,
    pub cheats_used: bool,
    pub credits: u32,
    pub members: Vec<PartyMember>,
    pub party_xp: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Global<T> {
    name: String,
    value: T,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Globals {
    booleans: Vec<Global<bool>>,
    numbers: Vec<Global<u8>>,
    strings: Vec<Global<String>>,
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
#[derive(Debug, Clone, PartialEq)]
pub struct Save {
    pub game: Game,
    pub globals: Globals,
    pub nfo: Nfo,
    pub party_table: PartyTable,
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
            let file = read_file(&(path.to_owned() + "/" + name))
                .map_err(|_| sf!("Couldn't read GFF file {name}"))?;
            let gff = gff::read(&file)?;
            gffs.push(gff);
        }

        let erf_bytes = read_file(&(path.to_owned() + "/" + ERF_NAME))
            .map_err(|_| sf!("Couldn't read ERF file"))?;
        let erf = erf::read(&erf_bytes)?;
        let reader = read::SaveReader::new(game, &gffs[0], &gffs[1], &gffs[2], &erf);

        reader.process()
    }
}
