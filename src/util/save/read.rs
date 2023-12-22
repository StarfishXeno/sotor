use crate::formats::{
    erf::{self, ERF},
    gff::{self, FieldValue, Struct, GFF},
};

use super::{Save, SaveGlobals, SaveNfo};

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
    nfo: &'a GFF,
    globals: &'a GFF,
    party_table: &'a GFF,
    erf: &'a ERF,
}
impl<'a> SaveReader<'a> {
    pub fn new(nfo: &'a GFF, globals: &'a GFF, party_table: &'a GFF, erf: &'a ERF) -> Self {
        Self {
            nfo,
            globals,
            party_table,
            erf,
        }
    }

    pub fn process(self) -> Result<Save, String> {
        let nfo = self.read_nfo()?;
        let globals = self.read_globals()?;

        Ok(Save { nfo, globals })
    }

    fn read_nfo(&self) -> Result<SaveNfo, String> {
        let fields = &self.nfo.content.fields;

        Ok(SaveNfo {
            save_name: get_field!(fields, "SAVEGAMENAME")?,
            area_name: get_field!(fields, "AREANAME")?,
            last_module: get_field!(fields, "LASTMODULE")?,
            cheats_used: get_field!(fields, "CHEATUSED", unwrap_byte)? != 0,
            time_played: get_field!(fields, "TIMEPLAYED", unwrap_dword)?,
        })
    }

    fn read_globals(&self) -> Result<SaveGlobals, String> {
        let fields = &self.globals.content.fields;

        let types: &[&str] = &["Number", "Boolean", "String"];
        let mut names: Vec<Vec<_>> = Vec::with_capacity(types.len());
        // Strings are stored as a struct list and go into a separate vector
        let mut values = Vec::with_capacity(types.len() - 1);
        let mut string_values = &vec![];

        for tp in types {
            let name_list =
                if let Some(FieldValue::List(list)) = fields.get(&("Cat".to_owned() + tp)) {
                    list
                } else {
                    return Err(rf!("Globals: missing or invalid Cat{tp}"));
                };

            let val = fields.get(&("Val".to_owned() + tp));
            if let Some(FieldValue::Void(bytes)) = val {
                values.push(bytes);
            } else if let Some(FieldValue::List(list)) = val {
                string_values = list;
            } else {
                return Err(rf!("Globals: missing or invalid Val{tp}"));
            };

            names.push(
                name_list
                    .into_iter()
                    .map(|s| get_field!(s.fields, "Name").unwrap())
                    .collect(),
            );
        }

        let mut numbers: Vec<_> = names
            .remove(0)
            .into_iter()
            .zip(values[0].into_iter().map(|v| *v))
            .collect();

        let boolean_bytes: Vec<_> = values[1].into_iter().map(|b| b.reverse_bits()).collect();
        let mut booleans: Vec<_> = names
            .remove(0)
            .into_iter()
            .enumerate()
            .map(|(idx, name)| {
                let byte_idx = idx / 8;
                let bit_idx = idx % 8;
                let byte = boolean_bytes[byte_idx];
                let bit = byte & (1 << bit_idx);

                (name, bit != 0)
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
            .collect();

        booleans.sort_unstable_by(|(a, _), (b, _)| a.cmp(b));
        numbers.sort_unstable_by(|(a, _), (b, _)| a.cmp(b));
        strings.sort_unstable_by(|(a, _), (b, _)| a.cmp(b));

        Ok(SaveGlobals {
            booleans,
            numbers,
            strings,
        })
    }
}
