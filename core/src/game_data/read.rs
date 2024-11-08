use crate::{
    formats::{
        bif::Bif,
        gff::{Field, Gff},
        key::Key,
        tlk::Tlk,
        twoda::TwoDA,
        ReadResource, ResourceType,
    },
    game_data::{Appearance, Class, Feat, Item, Quest, QuestStage},
    util::{
        fs::{read_dir_dirs, read_dir_filemap, read_file},
        prefix_to_sort_suffix, prepare_item_name, SResult,
    },
    BaseItem, Data, ItemSlot, WeaponType,
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
    let mut ext = tp.to_extension();
    ext.insert(0, '.');
    for over in overrides {
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
    let Ok(mods) = read_dir_dirs(&workshop_dir) else {
        return (None, vec![]);
    };
    let mut overrides = vec![];
    let mut dialog_override = None;
    for d in mods {
        let Ok(map) = read_dir_filemap(&d.path.clone().into()) else {
            continue;
        };
        if let Some(over) = map.get("override") {
            overrides.push(PathBuf::from_iter([&d.path, over.as_str()]));
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

pub fn read_feats(
    twoda: TwoDA,
    tlk_bytes: &[u8],
    descr_field: &str,
    extra_filter: Option<&[&str]>,
) -> SResult<Vec<(Feat, bool)>> {
    let mut tmp = Vec::with_capacity(twoda.0.len());
    let mut str_refs = Vec::with_capacity(twoda.0.len() * 2);
    let mut idx = 0;
    for feat in twoda.0 {
        let id = *feat["_idx"].as_ref().unwrap().int_unwrap();
        let label = feat["label"]
            .as_ref()
            .map(|t| t.string_unwrap().clone())
            .unwrap_or_default();
        let name_ref = feat["name"].as_ref().map_or(-1, |t| *t.int_unwrap());
        let descr_ref = feat[descr_field].as_ref().map_or(-1, |t| *t.int_unwrap());
        if name_ref == -1 && label.is_empty() {
            // some nameless garbage
            continue;
        }
        let extra = extra_filter.map_or(false, |prefixes| {
            !prefixes.iter().any(|p| label.starts_with(p)) || name_ref == -1
        });

        tmp.push((idx, id as u16, label, extra));
        str_refs.push(to_str_ref(name_ref));
        str_refs.push(to_str_ref(descr_ref));
        idx += 1;
    }
    let mut tlk =
        Tlk::read(tlk_bytes, &str_refs).map_err(|err| format!("couldn't read strings: {err}"))?;

    let mut feats = Vec::with_capacity(tmp.len());
    for (idx, id, label, extra) in tmp {
        let name = mem::take(&mut tlk.strings[idx * 2]);
        let descr = mem::take(&mut tlk.strings[idx * 2 + 1]);
        let name = if name.is_empty() { label } else { name };
        // so that Flurry and Improved Flurry go after each other instead of strictly alphabetically
        let sorting_name =
            prefix_to_sort_suffix(&name, &["Improved ", "Advanced ", "Knight ", "Master "]);
        let feat = Feat {
            id,
            sorting_name,
            name,
            description: (!descr.is_empty()).then_some(descr),
        };
        feats.push((feat, extra));
    }
    feats.sort_unstable_by(|a, b| a.0.sorting_name.cmp(&b.0.sorting_name));

    Ok(feats)
}

pub fn read_classes(twoda: TwoDA, tlk_bytes: &[u8]) -> SResult<Vec<Class>> {
    let mut tmp = Vec::with_capacity(twoda.0.len());
    let mut str_refs = Vec::with_capacity(twoda.0.len());
    let mut idx = 0;
    for class in twoda.0 {
        let id = *class["_idx"].as_ref().unwrap().int_unwrap();
        let force_user = class["spellgaintable"].is_some();
        let Some(name_ref) = class["name"].as_ref().map(|t| *t.int_unwrap()) else {
            continue;
        };
        let Some(hit_die) = class["hitdie"].as_ref().map(|t| *t.int_unwrap()) else {
            continue;
        };
        let Some(force_die) = class["forcedie"].as_ref().map(|t| *t.int_unwrap()) else {
            continue;
        };

        tmp.push((idx, id, force_user, hit_die as u8, force_die as u8));
        str_refs.push(to_str_ref(name_ref));
        idx += 1;
    }
    let mut tlk =
        Tlk::read(tlk_bytes, &str_refs).map_err(|err| format!("couldn't read strings: {err}"))?;

    let mut classes = Vec::with_capacity(tmp.len());
    for (idx, id, force_user, hit_die, force_die) in tmp {
        classes.push(Class {
            id,
            force_user,
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
        let id = *appearance["_idx"].clone().unwrap().int_unwrap() as u16;
        let Some(name) = appearance[field].clone().map(|t| t.string_unwrap().clone()) else {
            continue;
        };

        appearances.push(Appearance { id, name });
    }
    appearances.sort_unstable_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    appearances
}

pub fn read_quests(mut journal: Gff, tlk_bytes: &[u8]) -> SResult<Vec<Quest>> {
    let mut list = journal.take("Categories", Field::list_take)?;
    let mut tmp = Vec::with_capacity(list.len());
    let mut str_refs = Vec::with_capacity(list.len() * 10);
    for quest in &mut list {
        let id = quest.take("Tag", Field::string_take)?;
        let name_ref = quest.take("Name", Field::loc_string_take)?.0 as usize;
        let mut stages_list = quest.take("EntryList", Field::list_take)?;
        let mut stages = Vec::with_capacity(stages_list.len());
        for stage in &mut stages_list {
            let id = stage.take("ID", Field::dword_take)? as i32;
            let end = stage.take("End", Field::word_take)? != 0;
            let descr_ref = stage.take("Text", Field::loc_string_take)?.0 as usize;

            stages.push((id, end, descr_ref));
            str_refs.push(descr_ref);
        }
        tmp.push((id.to_lowercase(), name_ref, stages));
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

pub fn read_base_items(items: TwoDA) -> HashMap<i32, BaseItem> {
    let mut base_items = HashMap::with_capacity(items.0.len());
    for item in items.0 {
        let id = *item["_idx"].as_ref().unwrap().int_unwrap();
        let label = item["label"]
            .as_ref()
            .map(|t| t.string_unwrap().clone())
            .unwrap_or_default();
        let Some(droid_or_human) = item["droidorhuman"].as_ref().map(|t| *t.int_unwrap()) else {
            continue;
        };
        let Some(slot) = item["equipableslots"].as_ref().map(|t| *t.int_unwrap()) else {
            continue;
        };
        let Ok(usable_by) = (droid_or_human as u8).try_into() else {
            continue;
        };

        let slot = match slot {
            0x00200 | 0x00208 => ItemSlot::Implant,
            0x00001 => ItemSlot::Head,
            0x00008 => ItemSlot::Gloves,
            0x00180 => ItemSlot::Arms,
            0x00002 => ItemSlot::Armor,
            0x00400 | 0x20400 => ItemSlot::Belt,
            0x00010 | 0x00030 => {
                let Some(ranged) = item["weapontype"].as_ref().map(|t| *t.int_unwrap() != 1) else {
                    println!("invalid weapon {label}");
                    continue;
                };
                let two_handed = slot == 0x00010;
                let tp = match (ranged, two_handed) {
                    (false, false) => WeaponType::MeleeOneHanded,
                    (false, true) => WeaponType::MeleeTwoHanded,
                    (true, false) => WeaponType::RangedOneHanded,
                    (true, true) => WeaponType::RangedTwoHanded,
                };

                ItemSlot::Weapon(tp)
            }
            _ => continue,
        };
        base_items.insert(
            id,
            BaseItem {
                id,
                label,
                usable_by,
                slot,
            },
        );
    }
    base_items
}

pub fn read_items(items: Vec<Gff>, tlk_bytes: &[u8]) -> SResult<Vec<Item>> {
    let mut tmp = Vec::with_capacity(items.len());
    let mut str_refs = Vec::with_capacity(items.len() * 2);
    for item in items {
        let tag = item.get("Tag", Field::string)?;
        let base_item = item.get("BaseItem", Field::int)?;
        let name_ref = item.get("LocalizedName", Field::loc_string)?.0 as usize;
        let descr_ref = item.get("DescIdentified", Field::loc_string)?.0 as usize;
        let stack_size = item.get("StackSize", Field::word)?;
        let charges = item.get("Charges", Field::byte)?;
        let upgrade_level = item.get("UpgradeLevel", Field::byte).ok();

        tmp.push((
            tag,
            base_item,
            name_ref,
            descr_ref,
            stack_size,
            charges,
            upgrade_level,
            item.content,
        ));
        str_refs.push(name_ref);
        str_refs.push(descr_ref);
    }
    let tlk =
        Tlk::read(tlk_bytes, &str_refs).map_err(|err| format!("couldn't read strings: {err}"))?;
    let mut map: HashMap<_, _> = str_refs.into_iter().zip(tlk.strings).collect();
    let mut items = Vec::with_capacity(tmp.len());

    for (tag, base_item, name_ref, descr_ref, stack_size, charges, upgrade_level, raw) in tmp {
        let name = mem::take(map.get_mut(&name_ref).unwrap());
        let name = prepare_item_name(&name);
        let descr = mem::take(map.get_mut(&descr_ref).unwrap());
        items.push(Item {
            id: tag.to_lowercase(),
            tag,
            base_item,
            stack_size,
            charges,
            name: (!name.is_empty()).then_some(name),
            description: (!descr.is_empty()).then_some(descr),
            upgrade_level,

            raw,
        });
    }
    items.sort_unstable_by(|a, b| {
        a.get_name()
            .to_lowercase()
            .cmp(&b.get_name().to_lowercase())
    });

    Ok(items)
}
