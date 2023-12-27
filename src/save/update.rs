use crate::formats::gff::{Field, Struct};

use super::{Global, Save, GLOBALS_TYPES};

fn global_names<T>(globals: &[Global<T>]) -> Vec<String> {
    globals.iter().map(|g| g.name.clone()).collect()
}
pub struct Updater<'a> {
    save: &'a mut Save,
}

impl<'a> Updater<'a> {
    pub fn new(save: &'a mut Save) -> Self {
        Self { save }
    }

    pub fn process(mut self) {
        self.write_nfo();
        self.write_globals();
        self.write_party_table();
    }

    fn write_nfo(&mut self) {
        let nfo = &mut self.save.nfo;
        let fields = &mut self.save.inner.nfo.content.fields;

        fields.insert(
            "SAVEGAMENAME".to_owned(),
            Field::String(nfo.save_name.clone()),
        );
        fields.insert("CHEATUSED".to_owned(), Field::Byte(nfo.cheats_used as u8));
    }

    fn write_globals(&mut self) {
        let globals = &self.save.globals;
        let fields = &mut self.save.inner.globals.content.fields;

        // remove the old data first
        for tp in GLOBALS_TYPES {
            fields.remove(&("Val".to_owned() + tp));
            fields.remove(&("Cat".to_owned() + tp));
        }

        // write the names of all the globals
        let pairs: Vec<_> = GLOBALS_TYPES
            .iter()
            .zip([
                global_names(&globals.numbers),
                global_names(&globals.booleans),
                global_names(&globals.strings),
            ])
            .collect();
        for (name, list) in pairs {
            let structs = list
                .into_iter()
                .map(|name| Struct::new([("Name".to_owned(), Field::String(name))].into()))
                .collect();
            fields.insert("Cat".to_owned() + name, Field::List(structs));
        }

        // NUMBERS
        let number_values = globals.numbers.iter().map(|g| g.value).collect();
        fields.insert("ValNumber".to_owned(), Field::Void(number_values));

        // BOOLEANS
        let mut boolean_values = Vec::with_capacity(globals.booleans.len() / 8 + 1);
        for (idx, field) in globals.booleans.iter().enumerate() {
            let byte_idx = idx / 8;
            let bit_idx = 7 - (idx % 8);

            if boolean_values.len() == byte_idx {
                boolean_values.push(0);
            }

            boolean_values[byte_idx] |= (field.value as u8) << bit_idx;
        }
        fields.insert("ValBoolean".to_owned(), Field::Void(boolean_values));

        // STRINGS
        let string_values = globals
            .strings
            .iter()
            .map(|g| Struct::new([("String".to_owned(), Field::String(g.value.clone()))].into()))
            .collect();
        fields.insert("ValString".to_owned(), Field::List(string_values));
    }

    fn write_party_table(&mut self) {
        let fields = &mut self.save.inner.party_table.content.fields;
        let pt = &self.save.party_table;

        let journal_list = pt
            .journal
            .iter()
            .map(|e| {
                Struct::new(
                    [
                        ("JNL_PlotID".to_owned(), Field::String(e.id.clone())),
                        ("JNL_State".to_owned(), Field::Int(e.state)),
                        ("JNL_Time".to_owned(), Field::Dword(e.time)),
                        ("JNL_Date".to_owned(), Field::Dword(e.date)),
                    ]
                    .into(),
                )
            })
            .collect();
        fields.insert("JNL_Entries".to_owned(), Field::List(journal_list));

        let members_list: Vec<_> = pt
            .members
            .iter()
            .map(|m| {
                Struct::new(
                    [
                        ("PT_MEMBER_ID".to_owned(), Field::Int(m.idx as i32)),
                        ("PT_IS_LEADER".to_owned(), Field::Byte(m.leader as u8)),
                    ]
                    .into(),
                )
            })
            .collect();
        fields.insert(
            "PT_NUM_MEMBERS".to_owned(),
            Field::Byte(members_list.len() as u8),
        );
        fields.insert("PT_MEMBERS".to_owned(), Field::List(members_list));

        let av_members_list = pt
            .available_members
            .iter()
            .map(|m| {
                Struct::new(
                    [
                        ("PT_NPC_AVAIL".to_owned(), Field::Byte(m.available as u8)),
                        ("PT_NPC_SELECT".to_owned(), Field::Byte(m.selectable as u8)),
                    ]
                    .into(),
                )
            })
            .collect();
        fields.insert("PT_AVAIL_NPCS".to_owned(), Field::List(av_members_list));

        fields.insert(
            "PT_CHEAT_USED".to_owned(),
            Field::Byte(self.save.nfo.cheats_used as u8),
        );
        fields.insert("PT_GOLD".to_owned(), Field::Dword(pt.credits));
        fields.insert("PT_XP_POOL".to_owned(), Field::Int(pt.party_xp));
    }
}
