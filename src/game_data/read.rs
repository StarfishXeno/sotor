use crate::{
    formats::{
        bif::Bif,
        key::Key,
        tlk::Tlk,
        twoda::{TwoDA, TwoDAType, TwoDAValue},
        ReadResource, ResourceType,
    },
    game_data::Feat,
    util::{read_dir_dirs, read_dir_filemap, read_file, SResult},
};
use ahash::{HashMap, HashMapExt as _};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub const TWODAS: &[(&str, &[(&str, TwoDAType)])] = &[
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
    ("classes", &[("name", TwoDAType::Int)]),
    ("portraits", &[("baseresref", TwoDAType::String)]),
    ("appearance", &[("label", TwoDAType::String)]),
    ("soundset", &[("label", TwoDAType::String)]),
];

#[derive(Debug)]
pub enum ResourceSource {
    File(PathBuf),
    Bif { file: PathBuf, res_idx: u32 },
}

pub fn find_source(
    overrides: &[PathBuf],
    key: &Key,
    name: &str,
    tp: ResourceType,
) -> Option<ResourceSource> {
    for over in overrides {
        let mut path = over.clone();
        path.push(name);
        path.set_extension(tp.to_extension());
        if path.exists() {
            return Some(ResourceSource::File(path));
        }
    }

    let res_ref = key.resources.get(&(name, tp).into())?;
    let file = key.get_file_path(res_ref.file_idx);
    Some(ResourceSource::Bif {
        file,
        res_idx: res_ref.resource_idx,
    })
}

pub fn find_sources_by_name(
    overrides: &[PathBuf],
    key: &Key,
    names: &[&str],
    tp: ResourceType,
) -> SResult<Vec<ResourceSource>> {
    let mut sources = Vec::with_capacity(names.len());
    for name in names {
        let Some(source) = find_source(overrides, key, name, tp) else {
            return Err(format!("couldn't find resource {name}"));
        };
        sources.push(source);
    }

    Ok(sources)
}

pub fn find_sources_by_type(
    overrides: &[PathBuf],
    key: &Key,
    tp: ResourceType,
) -> Vec<ResourceSource> {
    // search in Key
    let mut sources = vec![];
    for (_, v) in key.resources.iter().filter(|(k, _)| k.1 == tp) {
        sources.push(ResourceSource::Bif {
            file: key.get_file_path(v.file_idx),
            res_idx: v.resource_idx,
        });
    }

    // search loose files in override
    for over in overrides {
        let mut ext = tp.to_extension();
        ext.insert(0, '.');
        let files = read_dir_filemap(over).unwrap_or_default();

        for (_, v) in files.iter().filter(|(k, _)| k.ends_with(&ext)) {
            sources.push(ResourceSource::File(PathBuf::from_iter([
                over.clone(),
                v.into(),
            ])));
        }
    }

    sources
}

pub fn get_resources<'a, T: ReadResource<'a, Arg>, Arg: 'a + Copy>(
    dir: &Path,
    sources: Vec<ResourceSource>,
    args: &[Arg],
) -> SResult<Vec<T>> {
    let mut in_files = vec![];
    // accumulating so we can read everything in the bif in 1 go
    let mut in_bif = HashMap::new();
    for (idx, source) in sources.into_iter().enumerate() {
        match source {
            ResourceSource::File(path) => {
                in_files.push((idx, path));
            }
            ResourceSource::Bif { file, res_idx } => {
                let bif_resources = in_bif.entry(file).or_insert_with(Vec::new);
                bif_resources.push((idx, res_idx as usize));
            }
        }
    }

    let mut resource_bytes = vec![];

    for (idx, path) in in_files {
        let bytes = fs::read(&path)
            .map_err(|err| format!("couldn't read resource file {path:?}: {err}"))?;
        resource_bytes.push((idx, bytes));
    }

    for (file, resources) in in_bif {
        let bif_bytes =
            read_file(dir, &file).map_err(|err| format!("couldn't read bif {file:?}: {err}"))?;
        let (indices, res_indices): (Vec<_>, Vec<_>) = resources.into_iter().unzip();
        let bif = Bif::read(&bif_bytes, &res_indices)
            .map_err(|err| format!("couldn't read bif {file:?}: {err}"))?;

        for (idx, bytes) in bif.resources.into_iter().enumerate() {
            resource_bytes.push((indices[idx], bytes));
        }
    }
    resource_bytes.sort_unstable_by_key(|r| r.0);

    let mut resources = Vec::with_capacity(resource_bytes.len());
    for (idx, bytes) in resource_bytes {
        let resource = T::read(&bytes, args[idx])
            .map_err(|err| format!("couldn't read resource {idx}: {err}"))?;
        resources.push(resource);
    }

    Ok(resources)
}

pub fn get_resource<'a, T: ReadResource<'a, Arg>, Arg: 'a + Copy>(
    dir: &Path,
    source: ResourceSource,
    arg: Arg,
) -> SResult<T> {
    let bytes = match source {
        ResourceSource::File(path) => {
            fs::read(&path).map_err(|err| format!("couldn't read file {path:?}: {err}"))?
        }
        ResourceSource::Bif { file, res_idx } => {
            let bif_bytes = read_file(dir, &file)
                .map_err(|err| format!("couldn't read bif {file:?}: {err}"))?;
            let mut bif = Bif::read(&bif_bytes, &[res_idx as usize])
                .map_err(|err| format!("couldn't read bif {file:?}: {err}"))?;
            bif.resources.pop().unwrap()
        }
    };
    T::read(&bytes, arg)
}

pub fn read_workshop_dir(steam_dir: &str) -> (Option<PathBuf>, Vec<PathBuf>) {
    let workshop_dir = PathBuf::from_iter([steam_dir, "workshop", "content", "208580"]);
    let Ok(mods) = read_dir_dirs(workshop_dir) else {
        return (None, vec![]);
    };
    let mut overrides = vec![];
    let mut dialog_override = None;
    for d in mods {
        let Ok(map) = read_dir_filemap(&d.path.into()) else {
            continue;
        };
        if let Some(over) = map.get("override") {
            overrides.push(over.into());
        }
        if let Some(dialog) = map.get("dialog.tlk") {
            dialog_override = Some(dialog.into());
        }
    }

    (dialog_override, overrides)
}

fn to_str_ref(v: i32) -> usize {
    if v.is_negative() {
        u32::MAX as usize
    } else {
        v as usize
    }
}

pub fn read_feats(
    twoda: TwoDA,
    tlk_bytes: &[u8],
    description_field: &str,
) -> SResult<HashMap<u16, Feat>> {
    let mut feats_tmp = Vec::with_capacity(twoda.0.len());
    let mut str_refs = Vec::with_capacity(twoda.0.len() * 2);
    let mut idx = 0;
    for feat in twoda.0 {
        let id = feat["_idx"].clone().unwrap().unwrap_int();
        let label = feat["label"]
            .clone()
            .map(TwoDAValue::unwrap_string)
            .unwrap_or_default();
        let name_ref = feat["name"].clone().map_or(-1, TwoDAValue::unwrap_int);
        let descr_ref = feat[description_field]
            .clone()
            .map_or(-1, TwoDAValue::unwrap_int);
        if name_ref == -1 && label.is_empty() {
            // some namesless garbage
            continue;
        }

        feats_tmp.push((idx, id as u16, label));
        str_refs.push(to_str_ref(name_ref));
        str_refs.push(to_str_ref(descr_ref));
        idx += 1;
    }
    let tlk = Tlk::read(tlk_bytes, &str_refs)
        .map_err(|err| format!("couldn't read feat strings: {err}"))?;

    let mut feats = HashMap::new();
    for (idx, id, label) in feats_tmp {
        let name = &tlk.strings[idx * 2];
        let descr = &tlk.strings[idx * 2 + 1];
        feats.insert(
            id,
            Feat {
                name: if name.is_empty() { label } else { name.clone() },
                description: (!descr.is_empty()).then_some(descr.clone()),
            },
        );
    }
    Ok(feats)
}
