use crate::formats::{
    erf::Erf,
    gff::{Field, Gff},
};

use super::{Game, Global, Globals, JournalEntry, Nfo, PartyMember, PartyTable, Save};

macro_rules! rf {
    ($($t:tt)*) => {{
        format!("Save::read| {}", format!($($t)*))
    }};
}

macro_rules! get_field {
    ($map:expr, $field:literal, $method:tt) => {
        $map[$field]
            .clone()
            .$method()
            .ok_or(rf!("Invalid field {}", $field))
    };
    ($map:expr, $field:literal) => {
        get_field!($map, $field, unwrap_string)
    };
}

pub struct SaveReader<'a> {
    game: Game,
    nfo: &'a Gff,
    globals: &'a Gff,
    party_table: &'a Gff,
    erf: &'a Erf,
}
impl<'a> SaveReader<'a> {
    pub fn new(
        game: Game,
        nfo: &'a Gff,
        globals: &'a Gff,
        party_table: &'a Gff,
        erf: &'a Erf,
    ) -> Self {
        Self {
            game,
            nfo,
            globals,
            party_table,
            erf,
        }
    }

    pub fn process(self) -> Result<Save, String> {
        let mut nfo = self.read_nfo()?;
        let globals = self.read_globals()?;
        let mut party_table = self.read_party_table()?;

        // unifying the flag in case it's somehow out of sync
        nfo.cheats_used = nfo.cheats_used || party_table.cheats_used;
        party_table.cheats_used = nfo.cheats_used;

        Ok(Save {
            game: self.game,
            globals,
            nfo,
            party_table,
        })
    }

    fn read_nfo(&self) -> Result<Nfo, String> {
        let fields = &self.nfo.content.fields;

        Ok(Nfo {
            save_name: get_field!(fields, "SAVEGAMENAME")?,
            area_name: get_field!(fields, "AREANAME")?,
            last_module: get_field!(fields, "LASTMODULE")?,
            cheats_used: get_field!(fields, "CHEATUSED", unwrap_byte)? != 0,
            time_played: get_field!(fields, "TIMEPLAYED", unwrap_dword)?,
        })
    }

    fn read_globals(&self) -> Result<Globals, String> {
        let fields = &self.globals.content.fields;

        let types: &[&str] = &["Number", "Boolean", "String"];
        let mut names: Vec<Vec<_>> = Vec::with_capacity(types.len());
        // Strings are stored as a struct list and go into a separate vector
        let mut values = Vec::with_capacity(types.len() - 1);
        let mut string_values = &vec![];

        for tp in types {
            let Some(Field::List(name_list)) = fields.get(&("Cat".to_owned() + tp)) else {
                return Err(rf!("Globals: missing or invalid Cat{tp}"));
            };

            let val = fields.get(&("Val".to_owned() + tp));
            if let Some(Field::Void(bytes)) = val {
                values.push(bytes);
            } else if let Some(Field::List(list)) = val {
                string_values = list;
            } else {
                return Err(rf!("Globals: missing or invalid Val{tp}"));
            };

            names.push(
                name_list
                    .iter()
                    .map(|s| get_field!(s.fields, "Name").unwrap())
                    .collect(),
            );
        }

        let mut numbers: Vec<_> = names
            .remove(0)
            .into_iter()
            .zip(values[0].iter().copied())
            .map(|(name, value)| Global { name, value })
            .collect();

        let boolean_bytes: Vec<_> = values[1].iter().map(|b| b.reverse_bits()).collect();
        let mut booleans: Vec<_> = names
            .remove(0)
            .into_iter()
            .enumerate()
            .map(|(idx, name)| {
                let byte_idx = idx / 8;
                let bit_idx = idx % 8;
                let byte = boolean_bytes[byte_idx];
                let bit = byte & (1 << bit_idx);

                Global {
                    name,
                    value: bit != 0,
                }
            })
            .collect();

        let mut strings: Vec<_> = names
            .remove(0)
            .into_iter()
            .zip(
                string_values
                    .iter()
                    .map(|v| get_field!(v.fields, "String").unwrap()),
            )
            .map(|(name, value)| Global { name, value })
            .collect();

        booleans.sort_unstable_by(|a, b| a.name.cmp(&b.name));
        numbers.sort_unstable_by(|a, b| a.name.cmp(&b.name));
        strings.sort_unstable_by(|a, b| a.name.cmp(&b.name));

        Ok(Globals {
            booleans,
            numbers,
            strings,
        })
    }
    fn read_party_table(&self) -> Result<PartyTable, String> {
        let fields = &self.party_table.content.fields;
        let journal_list = get_field!(fields, "JNL_Entries", unwrap_list)?;
        let mut journal = Vec::with_capacity(journal_list.len());
        for entry in journal_list {
            journal.push(JournalEntry {
                id: get_field!(entry.fields, "JNL_PlotID")?,
                state: get_field!(entry.fields, "JNL_State", unwrap_int)?,
                time: get_field!(entry.fields, "JNL_Time", unwrap_dword)?,
                date: get_field!(entry.fields, "JNL_Date", unwrap_dword)?,
            });
        }
        let cheats_used = get_field!(fields, "PT_CHEAT_USED", unwrap_byte)? != 0;
        let credits = get_field!(fields, "PT_GOLD", unwrap_dword)?;
        let party_list = get_field!(fields, "PT_MEMBERS", unwrap_list)?;

        let party: Result<_, String> = party_list
            .into_iter()
            .map(|m| {
                Ok(PartyMember {
                    idx: get_field!(m.fields, "PT_MEMBER_ID", unwrap_int)? as usize,
                    is_leader: get_field!(m.fields, "PT_IS_LEADER", unwrap_byte)? != 0,
                })
            })
            .collect();
        let party = party?;

        let party_xp = get_field!(fields, "PT_XP_POOL", unwrap_int)?;

        Ok(PartyTable {
            journal,
            cheats_used,
            credits,
            party,
            party_xp,
        })
    }
}
