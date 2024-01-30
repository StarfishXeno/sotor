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
        shorten_string, Game, SResult,
    },
};
use ahash::HashMap;
use serde::{Deserialize, Serialize};
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
            ("spellgaintable", TwoDAType::String),
            ("hitdie", TwoDAType::Int),
            ("forcedie", TwoDAType::Int),
        ],
    ),
    ("portraits", &[("baseresref", TwoDAType::String)]),
    ("appearance", &[("label", TwoDAType::String)]),
    ("soundset", &[("label", TwoDAType::String)]),
];

pub trait Data<I> {
    fn get_id(&self) -> &I;
    fn get_name(&self) -> &str;
}

macro_rules! impl_data {
    ($type:ident, $id_type:ident, $id_field:tt) => {
        impl Data<$id_type> for $type {
            fn get_id(&self) -> &$id_type {
                &self.$id_field
            }
            fn get_name(&self) -> &str {
                &self.name
            }
        }
    };

    ($type:ident, $id_type:ident) => {
        impl_data!($type, $id_type, id);
    };
}

pub trait DataExt<I>: Data<I> {
    fn get_sorting_name(&self) -> &str;
    fn get_description(&self) -> Option<&str>;
}

macro_rules! impl_data_ext {
    ($type:ident, $id_type:ident) => {
        impl DataExt<$id_type> for $type {
            fn get_sorting_name(&self) -> &str {
                &self.sorting_name
            }
            fn get_description(&self) -> Option<&str> {
                self.description.as_ref().map(|d| d.as_str())
            }
        }
    };
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Feat {
    pub id: u16,
    pub name: String,
    pub sorting_name: String,
    pub description: Option<String>,
}
impl_data!(Feat, u16);
impl_data_ext!(Feat, u16);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Power {
    pub id: u16,
    pub name: String,
    pub sorting_name: String,
    pub description: Option<String>,
    pub extra: bool,
}
impl_data!(Power, u16);
impl_data_ext!(Power, u16);

impl From<(Feat, bool)> for Power {
    fn from(value: (Feat, bool)) -> Self {
        let f = value.0;
        Self {
            id: f.id,
            name: f.name,
            sorting_name: f.sorting_name,
            description: f.description,
            extra: value.1,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Class {
    pub id: i32,
    pub name: String,
    pub force_user: bool,
    pub hit_die: u8,
    pub force_die: u8,
}
impl_data!(Class, i32);
impl DataExt<i32> for Class {
    fn get_sorting_name(&self) -> &str {
        &self.name
    }
    fn get_description(&self) -> Option<&str> {
        None
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Appearance {
    pub id: u16,
    pub name: String,
}
impl_data!(Appearance, u16);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QuestStage {
    pub id: i32,
    pub description: String,
    pub end: bool,
}
impl QuestStage {
    pub fn get_name(&self, max_len: usize) -> String {
        let id_str = self.id.to_string();
        let prefix_len = id_str.len() + 2;
        id_str + ") " + &shorten_string(&self.description, max_len - prefix_len).replace('\n', " ")
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Quest {
    pub id: String,
    pub name: String,
    pub stages: BTreeMap<i32, QuestStage>,
}
impl_data!(Quest, String);

impl Quest {
    pub fn get_first_stage_id(&self) -> i32 {
        *self.stages.first_key_value().unwrap().0
    }
}

impl PartialEq for Quest {
    fn eq(&self, other: &Self) -> bool {
        self.id.cmp(&other.id).is_eq()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Item {
    pub res_ref: String,
    pub tag: String,
    pub name: String,
    pub identified: bool,
    pub description: String,
    pub stack_size: u16,
}
impl_data!(Item, String, res_ref);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameData {
    pub id: u64,
    pub feats: Vec<Feat>,
    pub powers: Vec<Power>,
    pub classes: Vec<Class>,
    pub portraits: Vec<Appearance>,
    pub appearances: Vec<Appearance>,
    pub soundsets: Vec<Appearance>,
    pub quests: Vec<Quest>,
    pub items: Vec<Item>,
}

impl GameData {
    pub fn read<P: AsRef<Path>>(game: Game, dir: P, steam_dir: Option<P>) -> SResult<Self> {
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
            let (dialog_override, overrides) = read_workshop_dir(steam_dir.unwrap().as_ref());
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
        let [feats, powers, classes, portraits, appearances, soundsets] =
            twodas.try_into().unwrap();

        let journal_source = find_source(&overrides, &key, "global", ResourceType::Jrl)
            .ok_or("couldn't find global.jrl")?;
        let journal: Gff = get_resource(&dir, journal_source, ())
            .map_err(|err| format!("couldn't read global.jrl: {err}"))?;

        let item_sources = find_sources_by_type(&overrides, &key, ResourceType::Uti);
        let item_count = item_sources.len();
        let items: Vec<Gff> = get_resources(&dir, item_sources, &vec![(); item_count])
            .map_err(|err| format!("couldn't read item: {err}"))?;

        let tlk_bytes = fs::read(&dialog_path)
            .map_err(|err| format!("couldn't read {dialog_path:?}: {err}"))?;

        let mut soundsets = read_appearances(soundsets, "label");
        if game == Game::Two && !soundsets.iter().any(|s| s.id == 85) {
            // for some reason they aren't in the 2da, the rest seems to be fine
            // TODO figure out why
            for (id, name) in [(85, "Player male"), (83, "Player female")] {
                soundsets.push(Appearance {
                    id,
                    name: name.to_owned(),
                });
            }
            soundsets.sort_unstable_by(|a, b| a.name.cmp(&b.name));
        }
        Ok(Self {
            id: fastrand::u64(..),
            feats: read_feats(feats, &tlk_bytes, "description", None)
                .map_err(|err| format!("couldn't read feats: {err}"))?
                .into_iter()
                .map(|(f, _)| f)
                .collect(),
            powers: read_feats(
                powers,
                &tlk_bytes,
                "spelldesc",
                Some(&["FORCE_POWER", "FORM_FORCE", "FORM_SABER"]),
            )
            .map_err(|err| format!("couldn't read powers: {err}"))?
            .into_iter()
            .map(Into::into)
            .collect(),
            classes: read_classes(classes, &tlk_bytes)
                .map_err(|err| format!("couldn't read classes: {err}"))?,
            portraits: read_appearances(portraits, "baseresref"),
            appearances: read_appearances(appearances, "label"),
            soundsets,
            quests: read_quests(journal, &tlk_bytes)
                .map_err(|err| format!("couldn't read journal: {err}"))?,
            items: read_items(items, &tlk_bytes)
                .map_err(|err| format!("couldn't read items: {err}"))?,
        })
    }
}

// we need to both iterate through sorted lists of data and look values up by id
// so we make a wrapper of gamedata with vecs copied into hashmaps
// costs a couple MB of RAM though
macro_rules! impl_game_data_mapped {
    ($([$s:ident, $id_type:ident, [$($field:tt,)+]],)+) => {
        #[derive(Debug)]
        pub struct GameDataMapped {
            $($(pub $field: HashMap<$id_type, $s>,)+)+
            pub inner: GameData,
        }
        impl From<GameData> for GameDataMapped {
            fn from(inner: GameData) -> GameDataMapped {
                GameDataMapped {
                    $($($field: inner.$field.clone().into_iter().map(|s| (s.get_id().clone(), s)).collect(),)+)+
                    inner,
                }
            }
        }
    };
}

impl_game_data_mapped!(
    [Feat, u16, [feats,]],
    [Power, u16, [powers,]],
    [Class, i32, [classes,]],
    [Appearance, u16, [portraits, appearances, soundsets,]],
    [Quest, String, [quests,]],
    [Item, String, [items,]],
);
