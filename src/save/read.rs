use crate::{
    save::{
        AvailablePartyMember, Character, Class, Game, Gender, Global, GlobalValue, JournalEntry,
        Nfo, PartyMember, PartyTable, Save, SaveInternals, GLOBALS_TYPES, NPC_RESOURCE_PREFIX,
    },
    util::{get_party, string_lowercase_map, SResult},
};
use ahash::HashMap;
use egui::TextureHandle;
use internal::{
    erf::Erf,
    gff::{get_field, Field, Gff, Struct},
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
            &last_module,
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
            save_name: s
                .fields
                .get("SAVEGAMENAME")
                .and_then(|f| f.clone().get_string())
                .unwrap_or_default(), // autosaves don't have this field
            area_name: get_field!(s, "AREANAME", get_string)?,
            last_module: get_field!(s, "LASTMODULE", get_string)?,
            cheat_used: get_field!(s, "CHEATUSED")? != 0,
            time_played: get_field!(s, "TIMEPLAYED", get_dword)?,
        })
    }

    fn read_globals(&self) -> SResult<Vec<Global>> {
        let s = &self.globals.content;

        let mut names: Vec<Vec<_>> = Vec::with_capacity(GLOBALS_TYPES.len());
        // Strings are stored as a struct list and go into a separate vector
        let mut values = Vec::with_capacity(GLOBALS_TYPES.len() - 1);

        for tp in GLOBALS_TYPES {
            let Some(Field::List(name_list)) = s.fields.get(&("Cat".to_owned() + tp)) else {
                return Err(format!("Globals: missing or invalid Cat{tp}"));
            };

            let val = s.fields.get(&("Val".to_owned() + tp));
            if let Some(Field::Void(bytes)) = val {
                values.push(bytes);
            } else {
                return Err(format!("Globals: missing or invalid Val{tp}"));
            };

            names.push(
                name_list
                    .iter()
                    .map(|s| get_field!(s, "Name", get_string).unwrap())
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

    fn read_party_table(&self) -> SResult<PartyTable> {
        let s = &self.party_table.content;
        let journal = get_field!(s, "JNL_Entries", get_list)?
            .into_iter()
            .map(|e| {
                Ok(JournalEntry {
                    id: get_field!(e, "JNL_PlotID", get_string)?,
                    stage: get_field!(e, "JNL_State", get_int)?,
                    time: get_field!(e, "JNL_Time", get_dword)?,
                    date: get_field!(e, "JNL_Date", get_dword)?,
                })
            })
            .collect::<SResult<_>>()?;

        let members = get_field!(s, "PT_MEMBERS", get_list)?
            .into_iter()
            .map(|m| {
                Ok(PartyMember {
                    idx: get_field!(m, "PT_MEMBER_ID", get_int)? as usize,
                    leader: get_field!(m, "PT_IS_LEADER")? != 0,
                })
            })
            .collect::<SResult<_>>()?;

        let available_members = get_field!(s, "PT_AVAIL_NPCS", get_list)?
            .into_iter()
            .map(|m| {
                Ok(AvailablePartyMember {
                    available: get_field!(m, "PT_NPC_AVAIL")? != 0,
                    selectable: get_field!(m, "PT_NPC_SELECT")? != 0,
                })
            })
            .collect::<SResult<_>>()?;

        let influence = get_field!(s, "PT_INFLUENCE", get_list)
            .ok()
            .map(|i| {
                i.into_iter()
                    .map(|m| get_field!(m, "PT_NPC_INFLUENCE", get_int))
                    .collect::<SResult<_>>()
            })
            .transpose()?;
        let party_xp = get_field!(s, "PT_XP_POOL", get_int)?;
        let cheat_used = get_field!(s, "PT_CHEAT_USED")? != 0;
        let credits = get_field!(s, "PT_GOLD", get_dword)?;
        let components = get_field!(s, "PT_ITEM_COMPONEN", get_dword).ok();
        let chemicals = get_field!(s, "PT_ITEM_CHEMICAL", get_dword).ok();

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
        last_module: &Gff,
        count: usize,
    ) -> SResult<(Vec<Character>, Vec<Struct>)> {
        let mut characters = Vec::with_capacity(count + 1);
        let mut structs = Vec::with_capacity(count + 1);

        let mut player_field = get_field!(last_module.content, "Mod_PlayerList", get_list)?;
        if player_field.is_empty() {
            return Err("Couldn't get player character struct".to_string());
        }
        let player = player_field.remove(0);

        characters.push(self.read_character(&player, count)?);
        structs.push(player);

        for idx in 0..count {
            let Some(name) = map.get(&(NPC_RESOURCE_PREFIX.to_owned() + &idx.to_string())) else {
                continue;
            };
            let gff = Gff::read(&self.erf.get(name, ResourceType::Utc).unwrap().content)
                .map_err(|err| format!("Couldn't read NPC GFF {idx}: {err}"))?;

            characters.push(
                self.read_character(&gff.content, idx)
                    .map_err(|err| format!("Error parsing character {idx}: {err}"))?,
            );
            structs.push(gff.content);
        }

        Ok((characters, structs))
    }

    fn read_character(&self, s: &Struct, idx: usize) -> SResult<Character> {
        let name = get_field!(s, "FirstName", get_loc_string)?
            .1
            .first()
            .map_or_else(
                || {
                    get_party(self.game)
                        .get(idx)
                        .map_or("REALLY missing name".to_owned(), ToString::to_string)
                },
                |v| v.content.clone(),
            );

        let attributes = [
            get_field!(s, "Str")?,
            get_field!(s, "Dex")?,
            get_field!(s, "Con")?,
            get_field!(s, "Int")?,
            get_field!(s, "Wis")?,
            get_field!(s, "Cha")?,
        ];

        let skills = get_field!(s, "SkillList", get_list)?
            .into_iter()
            .map(|s| get_field!(s, "Rank"))
            .collect::<SResult<Vec<_>>>()?
            .try_into()
            .map_err(|_| "Invalid skill list".to_string())?;

        let feats = get_field!(s, "FeatList", get_list)?
            .into_iter()
            .map(|s| get_field!(s, "Feat", get_word))
            .collect::<SResult<_>>()?;

        let classes = get_field!(s, "ClassList", get_list)?
            .iter()
            .map(Self::read_class)
            .collect::<SResult<Vec<_>>>()?;

        let gender = Gender::try_from(get_field!(s, "Gender")?)
            .map_err(|id| format!("Invalid gender {id}"))?;

        Ok(Character {
            idx,
            name,
            tag: get_field!(s, "Tag", get_string)?,
            hp: get_field!(s, "CurrentHitPoints", get_short)?,
            hp_max: get_field!(s, "MaxHitPoints", get_short)?,
            fp: get_field!(s, "ForcePoints", get_short)?,
            fp_max: get_field!(s, "MaxForcePoints", get_short)?,
            min_1_hp: get_field!(s, "Min1HP")? != 0,
            good_evil: get_field!(s, "GoodEvil")?,
            experience: get_field!(s, "Experience", get_dword)?,
            attributes,
            skills,
            feats,
            classes,
            gender,
            portrait: get_field!(s, "PortraitId", get_word)?,
            appearance: get_field!(s, "Appearance_Type", get_word)?,
            soundset: get_field!(s, "SoundSetFile", get_word)?,
        })
    }

    fn read_class(class: &Struct) -> SResult<Class> {
        let powers = get_field!(class, "KnownList0", get_list)
            .ok()
            .map(|list| {
                list.into_iter()
                    .map(|s| get_field!(s, "Spell", get_word))
                    .collect::<SResult<_>>()
            })
            .transpose()?
            .unwrap_or_default();

        Ok(Class {
            id: get_field!(class, "Class", get_int)?,
            level: get_field!(class, "ClassLevel", get_short)?,
            powers,
        })
    }
}
