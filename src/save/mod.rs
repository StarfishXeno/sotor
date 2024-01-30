use crate::{
    save::update::Updater,
    util::{load_tga, Game, SResult},
};
#[cfg(target_arch = "wasm32")]
use ahash::HashMap;
use egui::{Context, TextureHandle, TextureOptions};
use internal::{
    erf::{self, Erf},
    gff::{self, Field, Gff, Struct},
    ReadResourceNoArg as _,
};
use macros::{EnumFromInt, EnumToInt};
use std::{
    collections::VecDeque,
    fmt::{self},
};
#[cfg(not(target_arch = "wasm32"))]
use std::{fs, path::PathBuf};

mod read;
mod update;

const GLOBALS_TYPES: &[&str] = &["Number", "Boolean"];
const NPC_RESOURCE_PREFIX: &str = "availnpc";

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
    pub stage: i32,
    pub date: u32,
    pub time: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PartyTable {
    pub journal: Vec<JournalEntry>,
    pub cheat_used: bool,
    pub credits: u32,
    pub members: Vec<PartyMember>,
    pub available_members: Vec<AvailablePartyMember>,
    pub influence: Option<Vec<i32>>,
    pub party_xp: i32,
    pub components: Option<u32>,
    pub chemicals: Option<u32>,
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
    pub cheat_used: bool,
    pub time_played: u32,
}

#[derive(EnumFromInt, EnumToInt, Debug, Clone, PartialEq)]
#[repr(u8)]
pub enum Gender {
    Male = 0,
    Female = 1,
    Both = 2,
    Other = 3,
    None = 4,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Class {
    pub id: i32,
    pub level: i16,
    pub powers: Option<Vec<u16>>, // IDs
}

#[derive(Debug, Clone, PartialEq)]
pub struct Character {
    pub idx: usize,
    pub name: String,
    pub tag: String,
    pub hp: i16,
    pub hp_max: i16,
    pub fp: i16,
    pub fp_max: i16,
    pub min_1_hp: bool,
    pub good_evil: u8,
    pub experience: u32,
    pub attributes: [u8; 6], // STR, DEX, CON, INT, WIS, CHA
    pub skills: [u8; 8],     // ranks in order of the skill menu
    pub feats: Vec<u16>,     // IDs
    pub classes: Vec<Class>,
    pub gender: Gender,
    pub portrait: u16,
    pub appearance: u16,
    pub soundset: u16,
}

impl Character {
    pub fn get_name(&self) -> &str {
        if self.name.is_empty() {
            &self.tag
        } else {
            &self.name
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    tag: String,
    count: u16,
    charges: u8,
    new: bool,
    upgrades: u32,                   // in K1
    upgrade_slots: Option<[i32; 6]>, // in K2
    properties: Field,
}

#[derive(Debug, Clone, PartialEq)]
struct SaveInternals {
    nfo: Gff,
    globals: Gff,
    party_table: Gff,
    erf: Erf,
    pifo: Option<Gff>,
    use_pifo: bool,
    characters: Vec<Struct>,
}
#[derive(Clone, PartialEq)]
pub struct Save {
    pub id: u64,
    pub game: Game,
    pub globals: Vec<Global>,
    pub nfo: Nfo,
    pub party_table: PartyTable,
    pub image: Option<TextureHandle>,
    pub characters: Vec<Character>,
    pub inventory: Vec<Item>,

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

impl Save {
    fn read(mut gffs: VecDeque<Gff>, erf: Erf, image: Option<TextureHandle>) -> SResult<Save> {
        let reader = read::Reader::new(
            gffs.pop_front().unwrap(),
            gffs.pop_front().unwrap(),
            gffs.pop_front().unwrap(),
            erf,
            gffs.pop_front(),
            image,
        );

        reader
            .into_save()
            .map_err(|err| format!("Save::read| {err}"))
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Save {
    pub fn read_from_directory(path: &str, ctx: &Context) -> SResult<Self> {
        use crate::util::{read_dir_filemap, read_file};
        // first let's find the files and map names to lowercase
        let file_names = read_dir_filemap(&path.into())
            .map_err(|err| format!("couldn't read dir {path}: {err}"))?;

        // ERF
        let erf_name = file_names
            .get(ERF_NAME)
            .ok_or(format!("Couldn't find ERF file {ERF_NAME}"))?;
        let erf_bytes = read_file(path, erf_name)
            .map_err(|err| format!("couldn't read ERF file {erf_name}: {err}"))?;
        let erf = Erf::read(&erf_bytes)?;

        // GFFs
        let mut gffs = VecDeque::with_capacity(GFFS.len());
        for (required, name) in GFFS {
            let Some(gff_name) = file_names.get(*name) else {
                if *required {
                    return Err(format!("couldn't find GFF file {name}"));
                }
                continue;
            };
            let file = read_file(path, gff_name)
                .map_err(|err| format!("couldn't read GFF file {gff_name}: {err}"))?;
            gffs.push_back(Gff::read(&file)?);
        }
        // autosaves don't have screenshots
        let image = file_names
            .get(IMAGE_NAME)
            .and_then(|image_name| {
                let file = fs::read(PathBuf::from_iter([path, image_name])).ok()?;
                load_tga(&file).ok()
            })
            .map(|tga| ctx.load_texture("save_image", tga, TextureOptions::NEAREST));

        Self::read(gffs, erf, image)
    }

    pub fn save_to_directory(path: &str, save: &mut Save) -> crate::util::ESResult {
        use crate::util::{backup_file, read_dir_filemap};

        Updater::new(save).update();
        let file_names = read_dir_filemap(&path.into())
            .map_err(|err| format!("couldn't read dir {path}: {err}"))?;

        for ((_, name), gff) in GFFS.iter().zip([
            Some(&save.inner.nfo),
            Some(&save.inner.globals),
            Some(&save.inner.party_table),
            save.inner.pifo.as_ref(),
        ]) {
            let Some(gff) = gff else {
                continue;
            };
            let gff_name = file_names.get(*name).map_or(*name, |n| n.as_str());
            let full_path = PathBuf::from_iter([path, gff_name]);
            let bytes = gff::write(gff.clone());

            backup_file(&full_path)
                .map_err(|err| format!("couldn't backup file {path:?}: {err}"))?;
            fs::write(full_path, &bytes)
                .map_err(|err| format!("Couldn't write GFF file {name}: {err}"))?;
        }

        let erf_name = file_names.get(ERF_NAME).map_or(ERF_NAME, |n| n.as_str());
        let full_path = PathBuf::from_iter([path, erf_name]);
        let bytes = erf::write(save.inner.erf.clone());

        backup_file(&full_path).map_err(|err| format!("couldn't backup file {path:?}: {err}"))?;
        fs::write(full_path, bytes).map_err(|err| format!("couldn't write ERF file: {err}"))?;

        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
impl Save {
    pub fn read_from_files(files: &HashMap<String, Vec<u8>>, ctx: &Context) -> SResult<Save> {
        // ERF
        let erf_bytes = files
            .get(ERF_NAME)
            .ok_or(format!("Couldn't find ERF file {ERF_NAME}"))?;
        let erf = Erf::read(erf_bytes)?;

        // GFFs
        let mut gffs = VecDeque::with_capacity(GFFS.len());
        for (required, name) in GFFS {
            let Some(bytes) = files.get(*name) else {
                if *required {
                    return Err(format!("couldn't find GFF file {name}"));
                }
                continue;
            };

            gffs.push_back(Gff::read(bytes)?);
        }
        let image = files
            .get(IMAGE_NAME)
            .and_then(|bytes| load_tga(bytes).ok())
            .map(|tga| ctx.load_texture("save_image", tga, TextureOptions::NEAREST));

        Self::read(gffs, erf, image)
    }

    pub fn save_to_zip(save: &mut Self) -> Vec<u8> {
        use std::io::{Cursor, Write};

        Updater::new(save).update();
        let buf = vec![];
        let mut zip = zip::ZipWriter::new(Cursor::new(buf));
        let options =
            zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

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

            zip.start_file(*name, options).unwrap();
            zip.write_all(&bytes).unwrap();
        }
        let erf_bytes = erf::write(save.inner.erf.clone());
        zip.start_file(ERF_NAME, options).unwrap();
        zip.write_all(&erf_bytes).unwrap();

        zip.finish().unwrap().into_inner()
    }

    pub fn reload(self) -> Self {
        let mut gffs =
            VecDeque::from_iter([self.inner.nfo, self.inner.globals, self.inner.party_table]);
        if let Some(pifo) = self.inner.pifo {
            gffs.push_back(pifo);
        };
        Self::read(gffs, self.inner.erf, self.image).unwrap()
    }
}
