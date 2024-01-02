use crate::{
    formats::{
        erf,
        gff::{self, Field, Struct},
    },
    save::{
        AvailablePartyMember, Character, Class, Game, Gender, Global, GlobalValue, JournalEntry,
        Nfo, PartyMember, PartyTable, Save, SaveInternals, GLOBALS_TYPES,
    },
    util::string_lowercase_map,
};
use egui::TextureHandle;

macro_rules! rf {
    ($($t:tt)*) => {{
        format!("Save::read| {}", format!($($t)*))
    }};
}

macro_rules! get_field {
    ($map:expr, $field:literal, $method:tt) => {
        $map.get($field)
            .ok_or(rf!("Missing field {}", $field))
            .and_then(|f| f.clone().$method().ok_or(rf!("Invalid field {}", $field)))
    };
    ($map:expr, $field:literal) => {
        get_field!($map, $field, unwrap_byte)
    };
}

pub struct Reader {
    inner: SaveInternals,
    game: Game,
    image: Option<TextureHandle>,
}

impl Reader {
    pub fn new(inner: SaveInternals, image: Option<TextureHandle>) -> Self {
        let game = if inner.erf.resources.get("pc").is_none() {
            Game::One
        } else {
            Game::Two
        };
        Self { inner, game, image }
    }

    pub fn into_save(self) -> Result<Save, String> {
        let mut nfo = self.read_nfo()?;
        let globals = self.read_globals()?;
        let mut party_table = self.read_party_table()?;
        let characters =
            self.read_characters(party_table.available_members.len(), &nfo.last_module)?;

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
            inner: self.inner,
        })
    }

    fn read_nfo(&self) -> Result<Nfo, String> {
        let fields = &self.inner.nfo.content.fields;

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
        let fields = &self.inner.globals.content.fields;

        let mut names: Vec<Vec<_>> = Vec::with_capacity(GLOBALS_TYPES.len());
        // Strings are stored as a struct list and go into a separate vector
        let mut values = Vec::with_capacity(GLOBALS_TYPES.len() - 1);

        for tp in GLOBALS_TYPES {
            let Some(Field::List(name_list)) = fields.get(&("Cat".to_owned() + tp)) else {
                return Err(rf!("Globals: missing or invalid Cat{tp}"));
            };

            let val = fields.get(&("Val".to_owned() + tp));
            if let Some(Field::Void(bytes)) = val {
                values.push(bytes);
            } else {
                return Err(rf!("Globals: missing or invalid Val{tp}"));
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
        let fields = &self.inner.party_table.content.fields;
        let journal_list = get_field!(fields, "JNL_Entries", unwrap_list)?;
        let mut journal = Vec::with_capacity(journal_list.len());
        for entry in journal_list {
            journal.push(JournalEntry {
                id: get_field!(entry.fields, "JNL_PlotID", unwrap_string)?,
                state: get_field!(entry.fields, "JNL_State", unwrap_int)?,
                time: get_field!(entry.fields, "JNL_Time", unwrap_dword)?,
                date: get_field!(entry.fields, "JNL_Date", unwrap_dword)?,
            });
        }

        let members_list = get_field!(fields, "PT_MEMBERS", unwrap_list)?;
        let members: Result<_, String> = members_list
            .into_iter()
            .map(|m| {
                Ok(PartyMember {
                    idx: get_field!(m.fields, "PT_MEMBER_ID", unwrap_int)? as usize,
                    leader: get_field!(m.fields, "PT_IS_LEADER")? != 0,
                })
            })
            .collect();
        let members = members?;

        let av_members_list = get_field!(fields, "PT_AVAIL_NPCS", unwrap_list)?;
        let available_members: Result<_, String> = av_members_list
            .into_iter()
            .map(|m| {
                Ok(AvailablePartyMember {
                    available: get_field!(m.fields, "PT_NPC_AVAIL")? != 0,
                    selectable: get_field!(m.fields, "PT_NPC_SELECT")? != 0,
                })
            })
            .collect();
        let available_members = available_members?;

        let party_xp = get_field!(fields, "PT_XP_POOL", unwrap_int)?;
        let cheats_used = get_field!(fields, "PT_CHEAT_USED")? != 0;
        let credits = get_field!(fields, "PT_GOLD", unwrap_dword)?;
        let (components, chemicals) = match self.game {
            Game::One => (0, 0),
            Game::Two => (
                get_field!(fields, "PT_ITEM_COMPONEN", unwrap_dword)?,
                get_field!(fields, "PT_ITEM_CHEMICAL", unwrap_dword)?,
            ),
        };

        Ok(PartyTable {
            journal,
            cheats_used,
            credits,
            members,
            available_members,
            party_xp,
            components,
            chemicals,
        })
    }

    fn read_characters(
        &self,
        count: usize,
        last_module: &str,
    ) -> Result<Vec<Option<Character>>, String> {
        const NPC_RESOURCE_PREFIX: &str = "availnpc";

        let resources = &self.inner.erf.resources;
        let mut characters = Vec::with_capacity(count + 1);
        let keys: Vec<_> = resources.keys().cloned().collect();
        // goddammit the casing is all over the place even in erf
        let map = string_lowercase_map(&keys);

        for idx in 0..count {
            let Some(name) = map.get(&(NPC_RESOURCE_PREFIX.to_owned() + &idx.to_string())) else {
                characters.push(None);
                continue;
            };
            let gff = gff::read(&resources[name].content)
                .map_err(|err| rf!("Couldn't read NPC GFF {idx}: {err}"))?;

            characters.push(Some(
                Self::read_character(&gff.content, idx)
                    .map_err(|err| rf!("Error parsing character {idx}: {err}"))?,
            ));
        }

        let module = {
            if let Some(name) = map.get(&last_module.to_lowercase()) {
                let module = &resources[name];
                let module_erf = erf::read(&module.content)?;
                let module_inner = module_erf
                    .resources
                    .get("Module")
                    .ok_or(rf!("Couldn't get inner module resource"))?;

                gff::read(&module_inner.content)?
            } else if let Some(res) = &self.inner.pifo {
                res.clone()
            } else {
                return Err("Couldn't get last module resource".to_owned());
            }
        };

        let player_field = get_field!(module.content.fields, "Mod_PlayerList", unwrap_list)?;
        let player = player_field
            .get(0)
            .ok_or(rf!("Couldn't get player character struct"))?;

        characters.push(Some(Self::read_character(player, count)?));

        Ok(characters)
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
            .map_err(|id| rf!("Invalid gender {id}"))?;

        let feats = get_field!(fields, "FeatList", unwrap_list)?
            .into_iter()
            .map(|s| get_field!(s.fields, "Feat", unwrap_word))
            .collect::<Result<_, _>>()?;

        let skills = get_field!(fields, "SkillList", unwrap_list)?
            .into_iter()
            .map(|s| get_field!(s.fields, "Rank"))
            .collect::<Result<Vec<_>, _>>()?
            .try_into()
            .map_err(|_| rf!("Invalid skill list"))?;

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
