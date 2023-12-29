use crate::{
    formats::{
        erf::{self, Erf},
        gff::{self, Gff},
    },
    util::{load_tga, read_dir_filemap},
};
use egui::{Context, TextureHandle, TextureOptions};
use std::{fmt, fs, path::PathBuf};

mod read;
mod update;

const GLOBALS_TYPES: &[&str] = &["Number", "Boolean", "String"];

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
#[derive(Debug, Clone, PartialEq, Copy)]
#[repr(u8)]
pub enum Game {
    One = 0,
    Two,
}
impl Game {
    pub const GAME_COUNT: usize = 2;

    pub fn get_party_names(self) -> &'static [&'static str] {
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
            Self::One => PARTY_1,
            Self::Two => PARTY_2,
        }
    }

    pub fn get_exe_name(self) -> String {
        match self {
            Self::One => "swkotor.exe".to_owned(),
            Self::Two => "swkotor2.exe".to_owned(),
        }
    }

    pub fn to_idx(self) -> usize {
        self as usize
    }

    pub fn get_save_directories(self) -> Vec<PathBuf> {
        let mut paths = vec![];

        if self == Self::One {
            return paths;
        }
        if cfg!(target_os = "windows") {
            let Ok(app_data) = std::env::var("LocalAppData") else {
                return paths;
            };

            let path_end = PathBuf::from_iter(["LucasArts", "SWKotORII"]);
            let mut path = PathBuf::from(app_data);
            let mut path_x86 = path.clone();
            path.push("Program Files");
            path_x86.push("Program Files (x86)");
            path.push(path_end.clone());
            path_x86.push(path_end);

            paths.push(path);
            paths.push(path_x86);
        } else if cfg!(target_os = "linux") {
            let Ok(home) = std::env::var("HOME") else {
                return paths;
            };
            let path = PathBuf::from_iter([&home, ".local", "share", "aspyr-media", "kotor2"]);

            paths.push(path);
        }

        paths
    }
}

#[derive(Debug, Clone, PartialEq)]
struct SaveInternals {
    nfo: Gff,
    globals: Gff,
    party_table: Gff,
    erf: Erf,
}
#[derive(Clone, PartialEq)]
pub struct Save {
    pub game: Game,
    pub globals: Globals,
    pub nfo: Nfo,
    pub party_table: PartyTable,
    pub image: TextureHandle,

    inner: SaveInternals,
}

// TextureHandle doesn't implement Debug and there isn't much point to printing internals
impl fmt::Debug for Save {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Resource")
            .field("game", &self.game)
            .field("globals", &self.globals)
            .field("nfo", &self.nfo)
            .field("party_table", &self.party_table)
            .finish_non_exhaustive()
    }
}

const GFF_NAMES: &[&str] = &["savenfo.res", "globalvars.res", "partytable.res"];
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
        let mut gffs = Vec::with_capacity(GFF_NAMES.len());
        for name in GFF_NAMES {
            let gff_name = file_names
                .get(*name)
                .ok_or(sf!("Couldn't find GFF file {name}"))?;
            let file = fs::read(PathBuf::from_iter([path, gff_name]))
                .map_err(|err| sf!("Couldn't read GFF file {gff_name}: {err}"))?;
            gffs.push(gff::read(&file)?);
        }

        // IMAGE
        let image_name = file_names
            .get(IMAGE_NAME)
            .ok_or(sf!("Couldn't find save image {IMAGE_NAME}"))?;
        let tga = load_tga(PathBuf::from_iter([path, image_name]))
            .map_err(|err| sf!("Couldn't read image {image_name}: {err}"))?;
        let texture = ctx.load_texture("save_image", tga, TextureOptions::NEAREST);

        let reader = read::Reader::new(
            SaveInternals {
                nfo: gffs.remove(0),
                globals: gffs.remove(0),
                party_table: gffs.remove(0),

                erf,
            },
            texture,
        );

        reader.into_save()
    }

    pub fn save_to_directory(path: &str, save: &mut Save) -> Result<(), String> {
        update::Updater::new(save).process();

        for (name, gff) in GFF_NAMES.iter().zip([
            &save.inner.nfo,
            &save.inner.globals,
            &save.inner.party_table,
        ]) {
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
