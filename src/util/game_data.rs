use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use crate::formats::{
    bif::Bif,
    gff::Gff,
    key::Key,
    twoda::{TwoDA, TwoDAType},
    ReadResource, ResourceType,
};

use super::{read_dir_filemap, read_file, SResult};

const TWODAS: &[(&str, &[(&str, TwoDAType)])] = &[
    ("feat", &[("name", TwoDAType::Int)]),
    ("spells", &[("name", TwoDAType::Int)]),
    ("classes", &[("name", TwoDAType::Int)]),
    ("portraits", &[("baseresref", TwoDAType::String)]),
    ("appearance", &[("label", TwoDAType::String)]),
    ("soundset", &[("label", TwoDAType::String)]),
];

#[derive(Debug)]
enum ResourceSource {
    File(PathBuf),
    Bif { file: PathBuf, res_idx: u32 },
}

fn find_source(
    over: &Option<PathBuf>,
    key: &Key,
    name: &str,
    tp: ResourceType,
) -> Option<ResourceSource> {
    let over_file = over.as_ref().and_then(|path| {
        let mut path = path.clone();
        path.push(name);
        path.set_extension(tp.to_extension());
        path.exists().then_some(path)
    });
    if let Some(over_file) = over_file {
        return Some(ResourceSource::File(over_file));
    }

    let res_ref = key.resources.get(&(name, tp).into())?;
    let file = key.get_file_path(res_ref.file_idx);
    Some(ResourceSource::Bif {
        file,
        res_idx: res_ref.resource_idx,
    })
}

fn find_sources_by_name(
    over: &Option<PathBuf>,
    key: &Key,
    names: &[&str],
    tp: ResourceType,
) -> SResult<Vec<ResourceSource>> {
    let mut sources = Vec::with_capacity(names.len());
    for name in names {
        let Some(source) = find_source(over, key, name, tp) else {
            return Err(format!("couldn't find resource {name}"));
        };
        sources.push(source);
    }

    Ok(sources)
}

fn find_sources_by_type(
    over: &Option<PathBuf>,
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

    let Some(over) = over else {
        return sources;
    };
    // search loose files in override
    let mut ext = tp.to_extension();
    ext.insert(0, '.');
    let files = read_dir_filemap(over).unwrap_or_default();

    for (_, v) in files.iter().filter(|(k, _)| k.ends_with(&ext)) {
        sources.push(ResourceSource::File(PathBuf::from_iter([
            over.clone(),
            v.into(),
        ])));
    }

    sources
}

fn get_resources<'a, T: ReadResource<'a, Arg>, Arg: 'a + Copy>(
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

fn get_resource<'a, T: ReadResource<'a, Arg>, Arg: 'a + Copy>(
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

pub fn read() {
    let dir: PathBuf = "/mnt/media/SteamLibrary/steamapps/common/swkotor".into();
    let map = read_dir_filemap(&dir).unwrap();
    let over = map.get("override").map(|over| {
        let mut dir = dir.clone();
        dir.push(over);
        dir
    });

    let key_bytes = read_file(&dir, "chitin.key").unwrap();
    let tlk_bytes = read_file(&dir, "dialog.tlk").unwrap();
    let key = Key::read(&key_bytes, ()).unwrap();

    let (twoda_names, twoda_args): (Vec<_>, Vec<_>) = TWODAS.iter().copied().unzip();
    let twoda_sources = find_sources_by_name(&over, &key, &twoda_names, TwoDA::get_type()).unwrap();
    let twodas: Vec<TwoDA> = get_resources(&dir, twoda_sources, &twoda_args).unwrap();
    let [feats, spells, classes, portraits, appearances, soundsets] = twodas.try_into().unwrap();

    let item_sources = find_sources_by_type(&over, &key, ResourceType::Uti);
    let item_count = item_sources.len();
    let items: Vec<Gff> = get_resources(&dir, item_sources, &[()].repeat(item_count)).unwrap();

    let journal_source = find_source(&over, &key, "global", ResourceType::Jrl).unwrap();
    let journal: Gff = get_resource(&dir, journal_source, ()).unwrap();
}
