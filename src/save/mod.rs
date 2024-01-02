use crate::{
    formats::{
        erf::{self, Erf},
        gff::{self, Gff},
    },
    util::{load_tga, read_dir_filemap},
};
use egui::{Context, TextureHandle, TextureOptions};
use sotor_macros::{EnumFromInt, EnumList};
use std::{
    collections::VecDeque,
    fmt::{self, Display},
    fs,
    path::PathBuf,
};

mod read;
mod update;

const GLOBALS_TYPES: &[&str] = &["Number", "Boolean"];

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
    pub components: u32,
    pub chemicals: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GlobalValue {
    Boolean(bool),
    Number(u8),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Global {
    pub name: String,
    pub value: GlobalValue,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Nfo {
    pub save_name: String,
    pub area_name: String,
    pub last_module: String,
    pub cheats_used: bool,
    pub time_played: u32,
}
#[derive(Debug, EnumList, Clone, PartialEq, Copy)]
#[repr(u8)]
pub enum Game {
    One = 0,
    Two,
}
impl Game {
    pub fn idx(self) -> usize {
        self as usize
    }
}
impl Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&(*self as u8 + 1).to_string())
    }
}

#[derive(EnumFromInt, Debug, Clone, PartialEq)]
#[repr(u8)]
enum Gender {
    Male = 0,
    Female = 1,
    Both = 2,
    Other = 3,
    None = 4,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Class {
    id: i32,
    level: i16,
    powers: Vec<u16>, // IDs
}

#[derive(Debug, Clone, PartialEq)]
pub struct Character {
    idx: usize,
    name: String,
    attributes: [u8; 6], // STR, DEX, CON, INT, WIS, CHA
    hp: i16,
    hp_max: i16,
    fp: i16,
    fp_max: i16,
    min_1_hp: bool,
    good_evil: u8,
    experience: u32,
    feats: Vec<u16>, // IDs
    skills: [u8; 8], // ranks in order of skill menu
    classes: Vec<Class>,

    gender: Gender,
}

#[derive(Debug, Clone, PartialEq)]
struct SaveInternals {
    nfo: Gff,
    globals: Gff,
    party_table: Gff,
    erf: Erf,
    pifo: Option<Gff>,
}
#[derive(Clone, PartialEq)]
pub struct Save {
    pub game: Game,
    pub globals: Vec<Global>,
    pub nfo: Nfo,
    pub party_table: PartyTable,
    pub image: Option<TextureHandle>,
    pub characters: Vec<Option<Character>>,

    inner: SaveInternals,
}

// TextureHandle doesn't implement Debug and there isn't much point to printing internals
impl fmt::Debug for Save {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Save")
            .field("game", &self.game)
            .field("globals", &self.globals)
            .field("nfo", &self.nfo)
            .field("party_table", &self.party_table)
            .finish_non_exhaustive()
    }
}

const GFFS: &[(bool, &str)] = &[
    (true, "savenfo.res"),
    (true, "globalvars.res"),
    (true, "partytable.res"),
    (false, "pifo.ifo"),
];
const ERF_NAME: &str = "savegame.sav";
const IMAGE_NAME: &str = "screen.tga";

macro_rules! sf {
    ($($t:tt)*) => {{
        format!("Save| {}", format!($($t)*))
    }};
}

impl Save {
    pub fn read_from_directory(path: &str, ctx: &Context) -> Result<Self, String> {
        // first let's find the files and map names to lowercase
        let file_names =
            read_dir_filemap(path.into()).map_err(|err| sf!("Couldn't read dir {path}: {err}"))?;

        // ERF
        let erf_name = file_names
            .get(ERF_NAME)
            .ok_or(sf!("Couldn't find ERF file {ERF_NAME}"))?;
        let erf_bytes = fs::read(PathBuf::from_iter([path, erf_name]))
            .map_err(|err| sf!("Couldn't read ERF file {erf_name}: {err}"))?;
        let erf = erf::read(&erf_bytes)?;

        // GFFs
        let mut gffs = VecDeque::with_capacity(GFFS.len());
        for (required, name) in GFFS {
            let Some(gff_name) = file_names.get(*name) else {
                if *required {
                    return Err(sf!("Couldn't find GFF file {name}"));
                }
                continue;
            };
            let file = fs::read(PathBuf::from_iter([path, gff_name]))
                .map_err(|err| sf!("Couldn't read GFF file {gff_name}: {err}"))?;
            gffs.push_back(gff::read(&file)?);
        }

        // autosaves don't have screenshots
        let texture = file_names
            .get(IMAGE_NAME)
            .and_then(|image_name| load_tga(PathBuf::from_iter([path, image_name])).ok())
            .map(|tga| ctx.load_texture("save_image", tga, TextureOptions::NEAREST));

        let reader = read::Reader::new(
            SaveInternals {
                nfo: gffs.pop_front().unwrap(),
                globals: gffs.pop_front().unwrap(),
                party_table: gffs.pop_front().unwrap(),
                pifo: gffs.pop_front(),

                erf,
            },
            texture,
        );

        reader.into_save()
    }

    pub fn save_to_directory(path: &str, save: &mut Save) -> Result<(), String> {
        update::Updater::new(save).process();

        for ((_, name), gff) in GFFS.iter().zip([
            Some(&save.inner.nfo),
            Some(&save.inner.globals),
            Some(&save.inner.party_table),
            save.inner.pifo.as_ref(),
        ]) {
            let Some(gff) = gff else {
                continue;
            };
            let bytes = gff::write(gff.clone());
            fs::write(PathBuf::from_iter([path, name]), &bytes)
                .map_err(|err| sf!("Couldn't write GFF file {name}: {}", err.to_string()))?;
        }

        fs::write(
            PathBuf::from_iter([path, ERF_NAME]),
            erf::write(save.inner.erf.clone()),
        )
        .map_err(|err| sf!("Couldn't write ERF file: {}", err.to_string()))?;

        Ok(())
    }
}
