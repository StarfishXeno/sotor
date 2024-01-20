use crate::{
    formats::{
        gff::Gff,
        key::Key,
        twoda::{TwoDA, TwoDAType},
        ReadResource, ResourceType,
    },
    game_data::read::{
        find_source, find_sources_by_name, find_sources_by_type, get_resource, get_resources,
        read_appearances, read_classes, read_feats, read_items, read_quests, read_workshop_dir,
    },
    util::{
        fs::{read_dir_filemap, read_file},
        Game, SResult,
    },
};
use ahash::HashMap;
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

mod read;

const TWODAS: &[(&str, &[(&str, TwoDAType)])] = &[
    (
        "feat",
        &[
            ("label", TwoDAType::String),
            ("name", TwoDAType::Int),
            ("description", TwoDAType::Int),
        ],
    ),
    (
        "spells",
        &[
            ("label", TwoDAType::String),
            ("name", TwoDAType::Int),
            ("spelldesc", TwoDAType::Int),
        ],
    ),
    (
        "classes",
        &[
            ("name", TwoDAType::Int),
            ("hitdie", TwoDAType::Int),
            ("forcedie", TwoDAType::Int),
        ],
    ),
    ("portraits", &[("baseresref", TwoDAType::String)]),
    ("appearance", &[("label", TwoDAType::String)]),
    ("soundset", &[("label", TwoDAType::String)]),
];

#[derive(Debug)]
pub struct Feat {
    name: String,
    description: Option<String>,
}

pub type Power = Feat;

#[derive(Debug)]
pub struct Class {
    name: String,
    hit_die: u8,
    force_die: u8,
}

#[derive(Debug)]
pub struct Appearance {
    name: String,
}

#[derive(Debug)]
pub struct QuestStage {
    description: String,
    end: bool,
}

#[derive(Debug)]
pub struct Quest {
    name: String,
    stages: BTreeMap<i32, QuestStage>,
}

#[derive(Debug)]
pub struct Item {
    tag: String,
    name: String,
    identified: bool,
    description: String,
    stack_size: u16,
}

#[derive(Debug)]
pub struct GameData {
    feats: HashMap<u16, Feat>,
    powers: HashMap<u16, Power>,
    classes: HashMap<i32, Class>,
    portraits: HashMap<u16, Appearance>,
    appearances: HashMap<u16, Appearance>,
    soundsets: HashMap<u16, Appearance>,
    quests: HashMap<String, Quest>,
    items: HashMap<String, Item>,
}

impl GameData {
    pub fn read(game: Game, dir: impl AsRef<Path>, steam_dir: Option<&str>) -> SResult<Self> {
        let mut dir: PathBuf = dir.as_ref().into();
        let mut map = read_dir_filemap(&dir)
            .map_err(|err| format!("couldn't read game dir {dir:?}: {err}"))?;
        // updated steam version of TSL stores game data in steamassets dir
        let steam_assets = map.get("steamassets");
        if game == Game::Two && steam_assets.is_some() {
            dir.push(steam_assets.unwrap());
            map = read_dir_filemap(&dir)
                .map_err(|err| format!("couldn't read steamassets dir {dir:?}: {err}"))?;
        }
        let mut dialog_path = dir.clone();
        dialog_path.push("dialog.tlk");

        let mut overrides = if game == Game::Two && steam_dir.is_some() {
            let (dialog_override, overrides) = read_workshop_dir(steam_dir.unwrap());
            if let Some(dialog) = dialog_override {
                dialog_path = dialog;
            }
            overrides
        } else {
            vec![]
        };
        // main dir override goes after workshop so it's of least priority
        if let Some(dir_override) = map.get("override") {
            let mut dir = dir.clone();
            dir.push(dir_override);
            overrides.push(dir);
        }

        let key_bytes = read_file(&dir, "chitin.key")
            .map_err(|err| format!("couldn't read chitin.key file: {err}"))?;
        let key = Key::read(&key_bytes, ()).map_err(|err| format!("couldn't read key: {err}"))?;

        let (twoda_names, twoda_args): (Vec<_>, Vec<_>) = TWODAS.iter().copied().unzip();
        let twoda_sources = find_sources_by_name(&overrides, &key, &twoda_names, TwoDA::get_type())
            .map_err(|err| format!("couldn't find 2da: {err}"))?;
        let twodas: Vec<TwoDA> = get_resources(&dir, twoda_sources, &twoda_args)
            .map_err(|err| format!("couldn't read 2da: {err}"))?;
        let [feats, spells, classes, portraits, appearances, soundsets] =
            twodas.try_into().unwrap();

        let journal_source = find_source(&overrides, &key, "global", ResourceType::Jrl)
            .ok_or("couldn't find global.jrl")?;
        let journal: Gff = get_resource(&dir, journal_source, ())
            .map_err(|err| format!("couldn't read global.jrl: {err}"))?;

        let item_sources = find_sources_by_type(&overrides, &key, ResourceType::Uti);
        let item_count = item_sources.len();
        let items: Vec<Gff> = get_resources(&dir, item_sources, &vec![(); item_count])
            .map_err(|err| format!("couldn't read item: {err}"))?;

        let tlk_bytes =
            fs::read(dialog_path).map_err(|err| format!("couldn't read dialog.tlk: {err}"))?;

        Ok(Self {
            feats: read_feats(feats, &tlk_bytes, "description")
                .map_err(|err| format!("couldn't read feats: {err}"))?,
            powers: read_feats(spells, &tlk_bytes, "spelldesc")
                .map_err(|err| format!("couldn't read powers: {err}"))?,
            classes: read_classes(classes, &tlk_bytes)
                .map_err(|err| format!("couldn't read classes: {err}"))?,
            portraits: read_appearances(portraits, "baseresref"),
            appearances: read_appearances(appearances, "label"),
            soundsets: read_appearances(soundsets, "label"),
            quests: read_quests(&journal, &tlk_bytes)
                .map_err(|err| format!("couldn't read journal: {err}"))?,
            items: read_items(&items, &tlk_bytes)
                .map_err(|err| format!("couldn't read items: {err}"))?,
        })
    }
}
