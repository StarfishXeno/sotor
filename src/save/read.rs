use crate::{
    save::{
        AvailablePartyMember, Character, Class, Game, Gender, Global, GlobalValue, JournalEntry,
        Nfo, PartyMember, PartyTable, Save, SaveInternals, GLOBALS_TYPES, NPC_RESOURCE_PREFIX,
    },
    util::{string_lowercase_map, SResult},
};
use ahash::HashMap;
use egui::TextureHandle;
use internal::{
    erf::Erf,
    gff::{Field, Gff, Struct},
    ReadResourceNoArg as _, ResourceType,
};
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
        let game = if erf.get("pc", ResourceType::Utc).is_none() {
            Game::One
        } else {
            Game::Two
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
        let resource_map = self.read_resource_map();
        let (use_pifo, last_module) = self.read_last_module(&resource_map, &nfo.last_module)?;
        let (characters, character_structs) = self.read_characters(
            &resource_map,
            last_module,
            party_table.available_members.len(),
        )?;

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

            inner: SaveInternals {
                nfo: self.nfo,
                globals: self.globals,
                party_table: self.party_table,
                erf: self.erf,
                pifo: self.pifo,

                use_pifo,
                characters: character_structs,
            },
        })
    }

    fn read_nfo(&self) -> SResult<Nfo> {
        let s = &self.nfo.content;

        Ok(Nfo {
            save_name: s.get("SAVEGAMENAME", Field::string).unwrap_or_default(), // autosaves don't have this field
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
            .get_ref("JNL_Entries", Field::list)?
            .iter()
            .map(|e| {
                Ok(JournalEntry {
                    id: e.get("JNL_PlotID", Field::string)?,
                    stage: e.get("JNL_State", Field::int)?,
                    time: e.get("JNL_Time", Field::dword)?,
                    date: e.get("JNL_Date", Field::dword)?,
                })
            })
            .collect::<SResult<_>>()?;

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

    // goddammit the casing is all over the place even in ERFs
    fn read_resource_map(&self) -> HashMap<String, String> {
        let resources = &self.erf.resources;
        let keys: Vec<_> = resources.keys().map(|k| k.0.clone()).collect();
        string_lowercase_map(&keys)
    }

    fn read_last_module(
        &self,
        map: &HashMap<String, String>,
        last_module: &str,
    ) -> SResult<(bool, Gff)> {
        if let Some(name) = map.get(&last_module.to_lowercase()) {
            let module = self.erf.get(name, ResourceType::Sav).unwrap();
            let module_erf = Erf::read(&module.content)?;
            let module_inner = module_erf
                .get("Module", ResourceType::Ifo)
                .ok_or("Couldn't get inner module resource".to_string())?;

            Ok((false, Gff::read(&module_inner.content)?))
        } else if let Some(res) = &self.pifo {
            Ok((true, res.clone()))
        } else {
            return Err("Couldn't get last module resource".to_string());
        }
    }

    fn read_characters(
        &self,
        map: &HashMap<String, String>,
        mut last_module: Gff,
        count: usize,
    ) -> SResult<(Vec<Character>, Vec<Struct>)> {
        let mut characters = Vec::with_capacity(count + 1);
        let mut structs = Vec::with_capacity(count + 1);

        let mut player_field = last_module.take("Mod_PlayerList", Field::list_take)?;
        if player_field.is_empty() {
            return Err("Couldn't get player character struct".to_string());
        }
        let player = player_field.remove(0);

        characters.push(Self::read_character(&player, count)?);
        structs.push(player);

        for idx in 0..count {
            let Some(name) = map.get(&(NPC_RESOURCE_PREFIX.to_owned() + &idx.to_string())) else {
                continue;
            };
            let gff = Gff::read(&self.erf.get(name, ResourceType::Utc).unwrap().content)
                .map_err(|err| format!("Couldn't read NPC GFF {idx}: {err}"))?;

            characters.push(
                Self::read_character(&gff.content, idx)
                    .map_err(|err| format!("Error parsing character {idx}: {err}"))?,
            );
            structs.push(gff.content);
        }

        Ok((characters, structs))
    }

    fn read_character(s: &Struct, idx: usize) -> SResult<Character> {
        let name = s
            .get_ref("FirstName", Field::loc_string)?
            .1
            .first()
            .map_or_else(String::new, |v| v.content.clone());

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
            .map_err(|id| format!("Invalid gender {id}"))?;

        Ok(Character {
            idx,
            name,
            tag: s.get("Tag", Field::string)?,
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
        })
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
}
