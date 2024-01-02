use std::collections::HashMap;

use crate::{
    formats::{
        erf::{self, Erf},
        gff::{self, Field, Gff, Struct},
    },
    save::{
        AvailablePartyMember, Character, Class, Game, Gender, Global, GlobalValue, JournalEntry,
        Nfo, PartyMember, PartyTable, Save, SaveInternals, GLOBALS_TYPES,
    },
    util::string_lowercase_map,
};
use egui::TextureHandle;

macro_rules! get_field {
    ($map:expr, $field:literal, $method:tt) => {
        $map.get($field)
            .ok_or(format!("Missing field {}", $field))
            .and_then(|f| {
                f.clone()
                    .$method()
                    .ok_or(format!("Invalid field {}", $field))
            })
    };
    ($map:expr, $field:literal) => {
        get_field!($map, $field, unwrap_byte)
    };
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
        let game = if erf.resources.get("pc").is_none() {
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

    pub fn into_save(self) -> Result<Save, String> {
        let mut nfo = self.read_nfo()?;
        let globals = self.read_globals()?;
        let mut party_table = self.read_party_table()?;
        let resource_map = self.read_resource_map();
        let (use_pifo, last_module) = self.read_last_module(&resource_map, &nfo.last_module)?;
        let (characters, character_structs) = self.read_characters(
            &resource_map,
            &last_module,
            party_table.available_members.len(),
        )?;

        // unifying the flag in case it's somehow out of sync
        nfo.cheats_used = nfo.cheats_used || party_table.cheats_used;
        party_table.cheats_used = nfo.cheats_used;

        Ok(Save {
            globals,
            nfo,
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

    fn read_nfo(&self) -> Result<Nfo, String> {
        let fields = &self.nfo.content.fields;

        Ok(Nfo {
            save_name: fields
                .get("SAVEGAMENAME")
                .and_then(|f| f.clone().unwrap_string())
                .unwrap_or_default(), // autosaves don't have this field
            area_name: get_field!(fields, "AREANAME", unwrap_string)?,
            last_module: get_field!(fields, "LASTMODULE", unwrap_string)?,
            cheats_used: get_field!(fields, "CHEATUSED")? != 0,
            time_played: get_field!(fields, "TIMEPLAYED", unwrap_dword)?,
        })
    }

    fn read_globals(&self) -> Result<Vec<Global>, String> {
        let fields = &self.globals.content.fields;

        let mut names: Vec<Vec<_>> = Vec::with_capacity(GLOBALS_TYPES.len());
        // Strings are stored as a struct list and go into a separate vector
        let mut values = Vec::with_capacity(GLOBALS_TYPES.len() - 1);

        for tp in GLOBALS_TYPES {
            let Some(Field::List(name_list)) = fields.get(&("Cat".to_owned() + tp)) else {
                return Err(format!("Globals: missing or invalid Cat{tp}"));
            };

            let val = fields.get(&("Val".to_owned() + tp));
            if let Some(Field::Void(bytes)) = val {
                values.push(bytes);
            } else {
                return Err(format!("Globals: missing or invalid Val{tp}"));
            };

            names.push(
                name_list
                    .iter()
                    .map(|s| get_field!(s.fields, "Name", unwrap_string).unwrap())
                    .collect(),
            );
        }

        let numbers: Vec<_> = names
            .remove(0)
            .into_iter()
            .zip(values[0].iter().copied())
            .map(|(name, value)| Global {
                name,
                value: GlobalValue::Number(value),
            })
            .collect();

        let boolean_bytes = values[1];
        let booleans: Vec<_> = names
            .remove(0)
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

        let mut globals: Vec<_> = [numbers, booleans].into_iter().flatten().collect();

        globals.sort_unstable_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

        Ok(globals)
    }

    fn read_party_table(&self) -> Result<PartyTable, String> {
        let fields = &self.party_table.content.fields;
        let journal = get_field!(fields, "JNL_Entries", unwrap_list)?
            .into_iter()
            .map(|e| {
                Ok(JournalEntry {
                    id: get_field!(e.fields, "JNL_PlotID", unwrap_string)?,
                    state: get_field!(e.fields, "JNL_State", unwrap_int)?,
                    time: get_field!(e.fields, "JNL_Time", unwrap_dword)?,
                    date: get_field!(e.fields, "JNL_Date", unwrap_dword)?,
                })
            })
            .collect::<Result<_, String>>()?;

        let members = get_field!(fields, "PT_MEMBERS", unwrap_list)?
            .into_iter()
            .map(|m| {
                Ok(PartyMember {
                    idx: get_field!(m.fields, "PT_MEMBER_ID", unwrap_int)? as usize,
                    leader: get_field!(m.fields, "PT_IS_LEADER")? != 0,
                })
            })
            .collect::<Result<_, String>>()?;

        let available_members = get_field!(fields, "PT_AVAIL_NPCS", unwrap_list)?
            .into_iter()
            .map(|m| {
                Ok(AvailablePartyMember {
                    available: get_field!(m.fields, "PT_NPC_AVAIL")? != 0,
                    selectable: get_field!(m.fields, "PT_NPC_SELECT")? != 0,
                })
            })
            .collect::<Result<_, String>>()?;

        let party_xp = get_field!(fields, "PT_XP_POOL", unwrap_int)?;
        let cheats_used = get_field!(fields, "PT_CHEAT_USED")? != 0;
        let credits = get_field!(fields, "PT_GOLD", unwrap_dword)?;
        let (components, chemicals, influence) = match self.game {
            Game::One => (None, None, None),
            Game::Two => {
                let influence = get_field!(fields, "PT_INFLUENCE", unwrap_list)?
                    .into_iter()
                    .map(|m| get_field!(m.fields, "PT_NPC_INFLUENCE", unwrap_int))
                    .collect::<Result<_, String>>()?;

                (
                    Some(get_field!(fields, "PT_ITEM_COMPONEN", unwrap_dword)?),
                    Some(get_field!(fields, "PT_ITEM_CHEMICAL", unwrap_dword)?),
                    Some(influence),
                )
            }
        };

        Ok(PartyTable {
            journal,
            cheats_used,
            credits,
            members,
            available_members,
            influence,
            party_xp,
            components,
            chemicals,
        })
    }

    // goddammit the casing is all over the place even in erf
    fn read_resource_map(&self) -> HashMap<String, String> {
        let resources = &self.erf.resources;
        let keys: Vec<_> = resources.keys().cloned().collect();
        string_lowercase_map(&keys)
    }

    fn read_last_module(
        &self,
        map: &HashMap<String, String>,
        last_module: &str,
    ) -> Result<(bool, Gff), String> {
        let resources = &self.erf.resources;

        if let Some(name) = map.get(&last_module.to_lowercase()) {
            let module = &resources[name];
            let module_erf = erf::read(&module.content)?;
            let module_inner = module_erf
                .resources
                .get("Module")
                .ok_or("Couldn't get inner module resource".to_string())?;

            Ok((false, gff::read(&module_inner.content)?))
        } else if let Some(res) = &self.pifo {
            Ok((true, res.clone()))
        } else {
            return Err("Couldn't get last module resource".to_string());
        }
    }

    fn read_characters(
        &self,
        map: &HashMap<String, String>,
        last_module: &Gff,
        count: usize,
    ) -> Result<(Vec<Character>, Vec<Struct>), String> {
        const NPC_RESOURCE_PREFIX: &str = "availnpc";
        let resources = &self.erf.resources;
        let mut characters = Vec::with_capacity(count + 1);
        let mut structs = Vec::with_capacity(count + 1);

        for idx in 0..count {
            let Some(name) = map.get(&(NPC_RESOURCE_PREFIX.to_owned() + &idx.to_string())) else {
                continue;
            };
            let gff = gff::read(&resources[name].content)
                .map_err(|err| format!("Couldn't read NPC GFF {idx}: {err}"))?;

            characters.push(
                Self::read_character(&gff.content, idx)
                    .map_err(|err| format!("Error parsing character {idx}: {err}"))?,
            );
            structs.push(gff.content);
        }

        let mut player_field =
            get_field!(last_module.content.fields, "Mod_PlayerList", unwrap_list)?;
        if player_field.is_empty() {
            return Err("Couldn't get player character struct".to_string());
        }
        let player = player_field.remove(0);

        characters.push(Self::read_character(&player, count)?);
        structs.push(player);

        Ok((characters, structs))
    }

    fn read_character(s: &Struct, idx: usize) -> Result<Character, String> {
        let fields = &s.fields;

        let name = get_field!(fields, "FirstName", unwrap_loc_string)?
            .1
            .get(0)
            .map_or("MISSING NAME".to_owned(), |v| v.content.clone());

        let attributes = [
            get_field!(fields, "Str")?,
            get_field!(fields, "Dex")?,
            get_field!(fields, "Con")?,
            get_field!(fields, "Int")?,
            get_field!(fields, "Wis")?,
            get_field!(fields, "Cha")?,
        ];

        let gender = Gender::try_from(get_field!(fields, "Gender")?)
            .map_err(|id| format!("Invalid gender {id}"))?;

        let feats = get_field!(fields, "FeatList", unwrap_list)?
            .into_iter()
            .map(|s| get_field!(s.fields, "Feat", unwrap_word))
            .collect::<Result<_, _>>()?;

        let skills = get_field!(fields, "SkillList", unwrap_list)?
            .into_iter()
            .map(|s| get_field!(s.fields, "Rank"))
            .collect::<Result<Vec<_>, _>>()?
            .try_into()
            .map_err(|_| "Invalid skill list".to_string())?;

        let classes = get_field!(fields, "ClassList", unwrap_list)?
            .iter()
            .map(Self::read_class)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Character {
            idx,
            name,
            attributes,
            hp: get_field!(fields, "HitPoints", unwrap_short)?,
            hp_max: get_field!(fields, "MaxHitPoints", unwrap_short)?,
            fp: get_field!(fields, "ForcePoints", unwrap_short)?,
            fp_max: get_field!(fields, "MaxForcePoints", unwrap_short)?,
            min_1_hp: get_field!(fields, "Min1HP")? != 0,
            good_evil: get_field!(fields, "GoodEvil")?,
            experience: get_field!(fields, "Experience", unwrap_dword)?,
            feats,
            skills,
            classes,
            gender,
        })
    }

    fn read_class(class: &Struct) -> Result<Class, String> {
        let powers = get_field!(class.fields, "KnownList0", unwrap_list)
            .ok()
            .map(|list| {
                list.into_iter()
                    .map(|s| get_field!(s.fields, "Spell", unwrap_word))
                    .collect::<Result<_, _>>()
            })
            .transpose()?
            .unwrap_or_default();

        Ok(Class {
            id: get_field!(class.fields, "Class", unwrap_int)?,
            level: get_field!(class.fields, "ClassLevel", unwrap_short)?,
            powers,
        })
    }
}
