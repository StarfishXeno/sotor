use crate::{
    save::{
        AvailablePartyMember, Character, Class, Door, Game, Gender, Global, GlobalValue, Item,
        JournalEntry, Nfo, PartyMember, PartyTable, Save, SaveInternals, EQUIPMENT_SLOT_IDS,
        GLOBALS_TYPES, NPC_RESOURCE_PREFIX,
    },
    util::{calc_hp_fp_offset, SResult},
};
use ahash::HashMap;
use core::{
    erf::Erf,
    gff::{Field, Gff, Struct},
    util::prepare_item_name,
    ReadResourceNoArg as _, ResourceKey, ResourceType,
};
use egui::TextureHandle;
use log::error;

struct LastModuleInfo {
    use_pifo: bool,
    module: Gff,
    git: Option<(ResourceKey, Gff)>,
}

pub struct Reader {
    nfo: Gff,
    globals: Gff,
    party_table: Gff,
    erf: Erf,
    pifo: Option<Gff>,
    game: Game,
    image: Option<TextureHandle>,
}

impl Reader {
    pub fn new(
        nfo: Gff,
        globals: Gff,
        party_table: Gff,
        erf: Erf,
        pifo: Option<Gff>,
        image: Option<TextureHandle>,
    ) -> Self {
        let game = if party_table.fields.get("PT_ITEM_CHEMICAL").is_some() {
            Game::Two
        } else {
            Game::One
        };

        Self {
            nfo,
            globals,
            party_table,
            erf,
            pifo,
            game,
            image,
        }
    }

    pub fn into_save(self) -> SResult<Save> {
        let mut nfo = self.read_nfo()?;
        let globals = self.read_globals()?;
        let mut party_table = self.read_party_table()?;
        let lm_info = self.read_last_module(&nfo.last_module.to_lowercase())?;
        let (characters, character_structs) =
            self.read_characters(lm_info.module, party_table.available_members.len())?;
        let inventory = self.read_inventory()?;
        let (git_key, doors) = match lm_info.git.map(|(key, gff)| (key, Self::read_doors(gff))) {
            Some((key, Some(d))) => (Some(key), Some(d)),
            _ => (None, None),
        };

        // unifying the flag in case it's somehow out of sync
        nfo.cheat_used = nfo.cheat_used || party_table.cheat_used;
        party_table.cheat_used = nfo.cheat_used;

        Ok(Save {
            id: fastrand::u64(..),
            nfo,
            globals,
            party_table,
            characters,
            image: self.image,
            game: self.game,
            inventory,
            doors,

            inner: SaveInternals {
                nfo: self.nfo,
                globals: self.globals,
                party_table: self.party_table,
                erf: self.erf,
                pifo: self.pifo,

                git_key,
                use_pifo: lm_info.use_pifo,
                characters: character_structs,
            },
        })
    }

    fn read_nfo(&self) -> SResult<Nfo> {
        let s = &self.nfo.content;

        Ok(Nfo {
            save_name: s.get("SAVEGAMENAME", Field::string).unwrap_or_default(), // autosaves don't have this field
            pc_name: s.get("PCNAME", Field::string).ok(),
            area_name: s.get("AREANAME", Field::string)?,
            last_module: s.get("LASTMODULE", Field::string)?,
            cheat_used: s.get("CHEATUSED", Field::byte)? != 0,
            time_played: s.get("TIMEPLAYED", Field::dword)?,
        })
    }

    fn read_globals(&self) -> SResult<Vec<Global>> {
        let s = &self.globals.content;

        let mut names: Vec<Vec<_>> = Vec::with_capacity(GLOBALS_TYPES.len());
        let mut values = Vec::with_capacity(GLOBALS_TYPES.len());

        for tp in GLOBALS_TYPES {
            let Some(Field::List(name_list)) = s.fields.get(&format!("Cat{tp}")) else {
                return Err(format!("Globals: missing or invalid Cat{tp}"));
            };

            if let Some(Field::Void(bytes)) = s.fields.get(&format!("Val{tp}")) {
                values.push(bytes);
            } else {
                return Err(format!("Globals: missing or invalid Val{tp}"));
            };

            names.push(
                name_list
                    .iter()
                    .map(|s| s.get("Name", Field::string))
                    .collect::<Result<_, _>>()?,
            );
        }

        let boolean_bytes = values[1];
        let booleans: Vec<_> = names
            .pop()
            .unwrap()
            .into_iter()
            .enumerate()
            .map(|(idx, name)| {
                let byte_idx = idx / 8;
                let bit_idx = 7 - idx % 8;
                let byte = boolean_bytes[byte_idx];
                let bit = byte & (1 << bit_idx);

                Global {
                    name,
                    value: GlobalValue::Boolean(bit != 0),
                }
            })
            .collect();

        let numbers: Vec<_> = names
            .pop()
            .unwrap()
            .into_iter()
            .zip(values[0].iter().copied())
            .map(|(name, value)| Global {
                name,
                value: GlobalValue::Number(value),
            })
            .collect();

        let mut globals: Vec<_> = [numbers, booleans].into_iter().flatten().collect();

        globals.sort_unstable_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

        Ok(globals)
    }

    fn read_party_table(&self) -> SResult<PartyTable> {
        let s = &self.party_table.content;
        let journal = s
            .get_ref("JNL_Entries", Field::list)
            .ok()
            .map(|list| {
                list.iter()
                    .map(|e| {
                        Ok(JournalEntry {
                            id: e.get("JNL_PlotID", Field::string)?,
                            stage: e.get("JNL_State", Field::int)?,
                            time: e.get("JNL_Time", Field::dword)?,
                            date: e.get("JNL_Date", Field::dword)?,
                        })
                    })
                    .collect::<SResult<_>>()
            })
            .transpose()?
            .unwrap_or_default();

        let members = s
            .get_ref("PT_MEMBERS", Field::list)?
            .iter()
            .map(|m| {
                Ok(PartyMember {
                    idx: m.get("PT_MEMBER_ID", Field::int)? as usize,
                    leader: m.get("PT_IS_LEADER", Field::byte)? != 0,
                })
            })
            .collect::<SResult<_>>()?;

        let available_members = s
            .get_ref("PT_AVAIL_NPCS", Field::list)?
            .iter()
            .map(|m| {
                Ok(AvailablePartyMember {
                    available: m.get("PT_NPC_AVAIL", Field::byte)? != 0,
                    selectable: m.get("PT_NPC_SELECT", Field::byte)? != 0,
                })
            })
            .collect::<SResult<_>>()?;

        let influence = s
            .get_ref("PT_INFLUENCE", Field::list)
            .ok()
            .map(|i| {
                i.iter()
                    .map(|m| m.get("PT_NPC_INFLUENCE", Field::int))
                    .collect::<SResult<_>>()
            })
            .transpose()?;
        let party_xp = s.get("PT_XP_POOL", Field::int)?;
        let cheat_used = s.get("PT_CHEAT_USED", Field::byte)? != 0;
        let credits = s.get("PT_GOLD", Field::dword)?;
        let components = s.get("PT_ITEM_COMPONEN", Field::dword).ok();
        let chemicals = s.get("PT_ITEM_CHEMICAL", Field::dword).ok();

        Ok(PartyTable {
            journal,
            cheat_used,
            credits,
            members,
            available_members,
            influence,
            party_xp,
            components,
            chemicals,
        })
    }

    fn read_last_module(&self, last_module: &str) -> SResult<LastModuleInfo> {
        if let Some(module) = self.erf.get(last_module, ResourceType::Sav) {
            let module_erf = Erf::read(&module.content)?;
            let module_inner = module_erf
                .get("module", ResourceType::Ifo)
                .ok_or("couldn't get inner module resource".to_string())?;
            let ifo = Gff::read(&module_inner.content)?;

            let mut git_key = None;
            for key in module_erf.resources.keys() {
                if last_module.ends_with(&key.0) && key.1 == ResourceType::Git {
                    git_key = Some(key.clone());
                    break;
                }
            }
            let module_git = git_key.as_ref().and_then(|k| module_erf.resources.get(k));
            let git = module_git.and_then(|g| Gff::read(&g.content).ok());

            Ok(LastModuleInfo {
                use_pifo: false,
                module: ifo,
                git: git_key.zip(git),
            })
        } else if let Some(res) = &self.pifo {
            Ok(LastModuleInfo {
                use_pifo: true,
                module: res.clone(),
                git: None,
            })
        } else {
            return Err("couldn't get last module resource".to_string());
        }
    }

    fn read_characters(
        &self,
        mut last_module: Gff,
        count: usize,
    ) -> SResult<(Vec<Character>, Vec<Struct>)> {
        let mut characters = Vec::with_capacity(count + 1);
        let mut structs = Vec::with_capacity(count + 1);

        let mut leader_field = last_module.take("Mod_PlayerList", Field::list_take)?;
        if leader_field.is_empty() {
            return Err("couldn't get player character struct".to_string());
        }
        let leader = leader_field.remove(0);

        characters.push(Self::read_character(&leader, usize::MAX)?);
        structs.push(leader);

        let mut keys = Vec::with_capacity(count + 1);
        // in case the PC isn't currently in the party
        keys.push(("pc".to_owned(), usize::MAX - 1));
        keys.extend((0..count).map(|idx| (format!("{NPC_RESOURCE_PREFIX}{idx}"), idx)));

        for (key, idx) in keys {
            let Some(resource) = self.erf.get(&key, ResourceType::Utc) else {
                continue;
            };
            let gff = Gff::read(&resource.content)
                .map_err(|err| format!("couldn't read NPC GFF {idx}: {err}"))?;
            let char = Self::read_character(&gff.content, idx)
                .map_err(|err| format!("error parsing character {idx}: {err}"))?;

            // this character currently leads the party and editing this second copy does nothing
            if char.tag == characters[0].tag {
                continue;
            }

            characters.push(char);
            structs.push(gff.content);
        }

        Ok((characters, structs))
    }

    fn read_character(s: &Struct, idx: usize) -> SResult<Character> {
        let nf = s.get_ref("FirstName", Field::loc_string)?;
        let name_ref = nf.0;
        let name = nf.1.first().map_or_else(String::new, |v| v.content.clone());
        let tag = s.get("Tag", Field::string)?;

        let attributes = [
            s.get("Str", Field::byte)?,
            s.get("Dex", Field::byte)?,
            s.get("Con", Field::byte)?,
            s.get("Int", Field::byte)?,
            s.get("Wis", Field::byte)?,
            s.get("Cha", Field::byte)?,
        ];

        let skills = s
            .get_ref("SkillList", Field::list)?
            .iter()
            .map(|s| s.get("Rank", Field::byte))
            .collect::<SResult<Vec<_>>>()?
            .try_into()
            .map_err(|_| "Invalid skill list".to_string())?;

        let feats = s
            .get_ref("FeatList", Field::list)?
            .iter()
            .map(|s| s.get("Feat", Field::word))
            .collect::<SResult<_>>()?;

        let classes = s
            .get_ref("ClassList", Field::list)?
            .iter()
            .map(Self::read_class)
            .collect::<SResult<Vec<_>>>()?;

        let gender = Gender::try_from(s.get("Gender", Field::byte)?)
            .map_err(|id| format!("invalid gender {id}"))?;

        let mut equipment = [(); 12].map(|_| None);
        let equipment_list = s.get_ref("Equip_ItemList", Field::list)?;
        let slot_map: HashMap<_, _> = EQUIPMENT_SLOT_IDS
            .into_iter()
            .enumerate()
            .map(|(idx, id)| (id, idx))
            .collect();

        for item in equipment_list {
            let slot = slot_map.get(&item.tp).ok_or_else(|| {
                format!("invalid item equipment slot {} on {name} {tag}", item.tp)
            })?;
            let item = Self::read_item(item)?;

            equipment[*slot] = Some(item);
        }
        let mut char = Character {
            idx,
            name,
            name_ref,
            tag,
            hp: s.get("CurrentHitPoints", Field::short)?,
            hp_max: s.get("MaxHitPoints", Field::short)?,
            fp: s.get("ForcePoints", Field::short)?,
            fp_max: s.get("MaxForcePoints", Field::short)?,
            min_1_hp: s.get("Min1HP", Field::byte)? != 0,
            good_evil: s.get("GoodEvil", Field::byte)?,
            experience: s.get("Experience", Field::dword)?,
            attributes,
            skills,
            feats,
            classes,
            gender,
            portrait: s.get("PortraitId", Field::word)?,
            appearance: s.get("Appearance_Type", Field::word)?,
            soundset: s.get("SoundSetFile", Field::word)?,
            equipment: Box::new(equipment),
        };
        let (hp, fp) = calc_hp_fp_offset(&char);
        char.hp += hp;
        char.fp += fp;

        Ok(char)
    }

    fn read_class(class: &Struct) -> SResult<Class> {
        let powers = class
            .get_ref("KnownList0", Field::list)
            .ok()
            .map(|list| {
                list.iter()
                    .map(|s| s.get("Spell", Field::word))
                    .collect::<SResult<_>>()
            })
            .transpose()?;

        Ok(Class {
            id: class.get("Class", Field::int)?,
            level: class.get("ClassLevel", Field::short)?,
            powers,
        })
    }

    fn read_inventory(&self) -> SResult<Vec<Item>> {
        let res = self
            .erf
            .get("inventory", ResourceType::Unknown)
            .ok_or("couldn't find inventory")?;
        let gff = Gff::read(&res.content)?;
        let list = gff.get_ref("ItemList", Field::list)?;
        let mut items = Vec::with_capacity(list.len());
        for item in list {
            items.push(Self::read_item(item)?);
        }

        Ok(items)
    }

    fn read_item(item: &Struct) -> SResult<Item> {
        let tag = item.get("Tag", Field::string)?;
        let name = item
            .get_ref("LocalizedName", Field::loc_string)?
            .1
            .first()
            .map(|s| prepare_item_name(s.content.as_str()));
        // only in K2
        let upgrade_slots = if let Ok(slot0) = item.get("UpgradeSlot0", Field::int) {
            Some([
                slot0,
                item.get("UpgradeSlot1", Field::int)?,
                item.get("UpgradeSlot2", Field::int)?,
                item.get("UpgradeSlot3", Field::int)?,
                item.get("UpgradeSlot4", Field::int)?,
                item.get("UpgradeSlot5", Field::int)?,
            ])
        } else {
            None
        };

        let item = Item {
            base_item: item.get("BaseItem", Field::int)?,
            name,
            description: item
                .get("DescIdentified", Field::loc_string)?
                .1
                .first()
                .map(|s| s.content.clone()),
            stack_size: item.get("StackSize", Field::word)?,
            max_charges: item.get("MaxCharges", Field::byte)?,
            charges: item.get("Charges", Field::byte)?,
            new: item.get("NewItem", Field::byte)? != 0,
            upgrades: item.get("Upgrades", Field::dword)?,
            upgrade_slots,
            raw: item.fields.clone(),
            tag,
        };

        Ok(item)
    }

    fn read_doors(mut gff: Gff) -> Option<Vec<Door>> {
        let list = gff.take("Door List", Field::list_take).ok()?;
        let mut doors = Vec::with_capacity(list.len());
        for door in list {
            let tag = door.get("Tag", Field::string).ok()?;
            let locked = door.get("Locked", Field::byte).ok()?;
            let open_state = door.get("OpenState", Field::byte).ok()?;

            // idk, just to be safe, i'm clamping the values from here on out
            if locked > 1 {
                error!("door locked value > 1");
                return None;
            }
            if open_state > 2 {
                error!("door open state value > 2");
                return None;
            }

            doors.push(Door {
                tag,
                locked: locked == 1,
                open_state,
                raw: door,
            });
        }

        Some(doors)
    }
}
