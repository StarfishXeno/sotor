use crate::{
    formats::{
        bif::Bif,
        gff::{Field, Gff},
        key::Key,
        tlk::Tlk,
        twoda::{TwoDA, TwoDAValue},
        ReadResource, ResourceType,
    },
    game_data::{Appearance, Class, Feat, Item, Quest, QuestStage},
    util::{
        fs::{read_dir_dirs, read_dir_filemap, read_file},
        SResult,
    },
};
use ahash::{HashMap, HashMapExt as _};
use std::{
    fs, mem,
    path::{Path, PathBuf},
};

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

pub fn read_workshop_dir(steam_dir: impl AsRef<Path>) -> (Option<PathBuf>, Vec<PathBuf>) {
    let workshop_dir = PathBuf::from_iter([
        &steam_dir.as_ref().to_string_lossy(),
        "workshop",
        "content",
        "208580",
    ]);
    let Ok(mods) = read_dir_dirs(workshop_dir) else {
        return (None, vec![]);
    };
    let mut overrides = vec![];
    let mut dialog_override = None;
    for d in mods {
        let Ok(map) = read_dir_filemap(&d.path.clone().into()) else {
            continue;
        };
        if let Some(over) = map.get("override") {
            overrides.push(over.into());
        }
        if let Some(dialog) = map.get("dialog.tlk") {
            dialog_override = Some(PathBuf::from_iter([&d.path, dialog.as_str()]));
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

pub fn read_feats(twoda: TwoDA, tlk_bytes: &[u8], description_field: &str) -> SResult<Vec<Feat>> {
    let mut tmp = Vec::with_capacity(twoda.0.len());
    let mut str_refs = Vec::with_capacity(twoda.0.len() * 2);
    let mut idx = 0;
    for feat in twoda.0 {
        let id = feat["_idx"].clone().unwrap().int_unwrap();
        let label = feat["label"]
            .clone()
            .map(TwoDAValue::string_unwrap)
            .unwrap_or_default();
        let name_ref = feat["name"].clone().map_or(-1, TwoDAValue::int_unwrap);
        let descr_ref = feat[description_field]
            .clone()
            .map_or(-1, TwoDAValue::int_unwrap);
        if name_ref == -1 && label.is_empty() {
            // some namesless garbage
            continue;
        }

        tmp.push((idx, id as u16, label));
        str_refs.push(to_str_ref(name_ref));
        str_refs.push(to_str_ref(descr_ref));
        idx += 1;
    }
    let mut tlk =
        Tlk::read(tlk_bytes, &str_refs).map_err(|err| format!("couldn't read strings: {err}"))?;

    let mut feats = Vec::with_capacity(tmp.len());
    for (idx, id, label) in tmp {
        let name = mem::take(&mut tlk.strings[idx * 2]);
        let descr = mem::take(&mut tlk.strings[idx * 2 + 1]);
        feats.push(Feat {
            id,
            name: if name.is_empty() { label } else { name },
            description: (!descr.is_empty()).then_some(descr),
        });
    }
    feats.sort_unstable_by(|a, b| a.name.cmp(&b.name));

    Ok(feats)
}

pub fn read_classes(twoda: TwoDA, tlk_bytes: &[u8]) -> SResult<Vec<Class>> {
    let mut tmp = Vec::with_capacity(twoda.0.len());
    let mut str_refs = Vec::with_capacity(twoda.0.len() * 2);
    let mut idx = 0;
    for class in twoda.0 {
        let id = class["_idx"].clone().unwrap().int_unwrap();
        let Some(name_ref) = class["name"].clone().map(TwoDAValue::int_unwrap) else {
            continue;
        };
        let Some(hit_die) = class["hitdie"].clone().map(TwoDAValue::int_unwrap) else {
            continue;
        };
        let Some(force_die) = class["forcedie"].clone().map(TwoDAValue::int_unwrap) else {
            continue;
        };

        tmp.push((idx, id, hit_die as u8, force_die as u8));
        str_refs.push(to_str_ref(name_ref));
        idx += 1;
    }
    let mut tlk =
        Tlk::read(tlk_bytes, &str_refs).map_err(|err| format!("couldn't read strings: {err}"))?;

    let mut classes = Vec::with_capacity(tmp.len());
    for (idx, id, hit_die, force_die) in tmp {
        classes.push(Class {
            id,
            name: mem::take(&mut tlk.strings[idx]),
            hit_die,
            force_die,
        });
    }
    classes.sort_unstable_by(|a, b| a.name.cmp(&b.name));

    Ok(classes)
}

pub fn read_appearances(twoda: TwoDA, field: &str) -> Vec<Appearance> {
    let mut appearances = Vec::with_capacity(twoda.0.len());
    for appearance in twoda.0 {
        let id = appearance["_idx"].clone().unwrap().int_unwrap() as u16;
        let Some(name) = appearance[field].clone().map(TwoDAValue::string_unwrap) else {
            continue;
        };

        appearances.push(Appearance { id, name });
    }

    appearances
}

pub fn read_quests(journal: &Gff, tlk_bytes: &[u8]) -> SResult<Vec<Quest>> {
    let list = journal.get("Categories", Field::list)?;
    let mut tmp = Vec::with_capacity(list.len());
    let mut str_refs = Vec::with_capacity(list.len() * 10);
    for quest in list {
        let id = quest.get("Tag", Field::string)?;
        let name_ref = quest.get("Name", Field::loc_string)?.0 as usize;
        let stages_list = quest.get("EntryList", Field::list)?;
        let mut stages = Vec::with_capacity(stages_list.len());
        for stage in stages_list {
            let id = stage.get("ID", Field::dword)? as i32;
            let end = stage.get("End", Field::word)? != 0;
            let descr_ref = stage.get("Text", Field::loc_string)?.0 as usize;

            stages.push((id, end, descr_ref));
            str_refs.push(descr_ref);
        }
        tmp.push((id, name_ref, stages));
        str_refs.push(name_ref);
    }
    let tlk =
        Tlk::read(tlk_bytes, &str_refs).map_err(|err| format!("couldn't read strings: {err}"))?;
    let mut map: HashMap<_, _> = str_refs.into_iter().zip(tlk.strings).collect();
    let mut quests = Vec::with_capacity(tmp.len());

    for (id, name_ref, stages) in tmp {
        let stages = stages
            .into_iter()
            .map(|(id, end, descr_ref)| {
                (
                    id,
                    QuestStage {
                        id,
                        end,
                        description: mem::take(map.get_mut(&descr_ref).unwrap()),
                    },
                )
            })
            .collect();

        quests.push(Quest {
            id,
            name: mem::take(map.get_mut(&name_ref).unwrap()),
            stages,
        });
    }
    quests.sort_unstable_by(|a, b| a.name.cmp(&b.name));

    Ok(quests)
}

pub fn read_items(items: &[Gff], tlk_bytes: &[u8]) -> SResult<Vec<Item>> {
    let mut tmp = Vec::with_capacity(items.len());
    let mut str_refs = Vec::with_capacity(items.len() * 2);
    for item in items {
        let s = &item.content;
        let tag = s.get("Tag", Field::string)?;
        let res_ref = s.get("TemplateResRef", Field::res_ref)?;
        let identified = s.get("Identified", Field::get_bool)?;
        let name_ref = s.get("LocalizedName", Field::loc_string)?.0 as usize;
        let descr_ref = s.get("DescIdentified", Field::loc_string)?.0 as usize;
        let stack_size = s.get("StackSize", Field::word)?;

        tmp.push((tag, res_ref, identified, name_ref, descr_ref, stack_size));
        str_refs.push(name_ref);
        str_refs.push(descr_ref);
    }
    let tlk =
        Tlk::read(tlk_bytes, &str_refs).map_err(|err| format!("couldn't read strings: {err}"))?;
    let mut map: HashMap<_, _> = str_refs.into_iter().zip(tlk.strings).collect();
    let mut items = Vec::with_capacity(tmp.len());

    for (tag, res_ref, identified, name_ref, descr_ref, stack_size) in tmp {
        items.push(Item {
            res_ref,
            tag,
            identified,
            stack_size,
            name: mem::take(map.get_mut(&name_ref).unwrap()),
            description: mem::take(map.get_mut(&descr_ref).unwrap()),
        });
    }
    items.sort_unstable_by(|a, b| a.name.cmp(&b.name));

    Ok(items)
}
