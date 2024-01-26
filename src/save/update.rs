use crate::{
    save::{Character, Class, GlobalValue, Save, GLOBALS_TYPES, NPC_RESOURCE_PREFIX},
    util::string_lowercase_map,
};
use internal::{
    erf::{self, Erf},
    gff::{self, Field, Gff, Struct},
    LocString, ReadResourceNoArg as _, ResourceType,
};

pub struct Updater<'a> {
    save: &'a mut Save,
}

impl<'a> Updater<'a> {
    pub fn new(save: &'a mut Save) -> Self {
        Self { save }
    }

    pub fn update(mut self) {
        self.update_nfo();
        self.update_globals();
        self.update_party_table();
        self.update_characters();
        self.update_pifo();
        self.update_erf();
    }

    fn update_nfo(&mut self) {
        let nfo = &mut self.save.nfo;
        let fields = &mut self.save.inner.nfo.content.fields;

        fields.insert(
            "SAVEGAMENAME".to_owned(),
            Field::String(nfo.save_name.clone()),
        );
        fields.insert("CHEATUSED".to_owned(), Field::Byte(nfo.cheat_used as u8));
        fields.insert(
            "PCNAME".to_owned(),
            Field::String(self.save.characters.first().unwrap().name.clone()),
        );
    }

    fn update_globals(&mut self) {
        let globals = &self.save.globals;
        let fields = &mut self.save.inner.globals.content.fields;
        let mut numbers = vec![];
        let mut booleans = vec![];
        for global in globals {
            let name = &global.name;
            match global.value {
                GlobalValue::Number(value) => numbers.push((name, value)),
                GlobalValue::Boolean(value) => booleans.push((name, value)),
            }
        }

        // write the names of all the globals
        let pairs: Vec<_> = GLOBALS_TYPES
            .iter()
            .zip([
                numbers.iter().map(|g| g.0.clone()).collect::<Vec<_>>(),
                booleans.iter().map(|g| g.0.clone()).collect(),
            ])
            .collect();

        for (name, list) in pairs {
            let structs = list
                .into_iter()
                .map(|name| Struct::new(vec![("Name", Field::String(name))]))
                .collect();
            fields.insert("Cat".to_owned() + name, Field::List(structs));
        }

        // NUMBERS
        let number_values = numbers.iter().map(|g| g.1).collect();
        fields.insert("ValNumber".to_owned(), Field::Void(number_values));

        // BOOLEANS
        let mut boolean_values = Vec::with_capacity(booleans.len() / 8 + 1);
        for (idx, field) in booleans.iter().enumerate() {
            let byte_idx = idx / 8;
            let bit_idx = 7 - (idx % 8);

            if boolean_values.len() == byte_idx {
                boolean_values.push(0);
            }

            boolean_values[byte_idx] |= (field.1 as u8) << bit_idx;
        }
        fields.insert("ValBoolean".to_owned(), Field::Void(boolean_values));
    }

    fn update_party_table(&mut self) {
        let fields = &mut self.save.inner.party_table.content.fields;
        let pt = &self.save.party_table;

        let journal_list = pt
            .journal
            .iter()
            .map(|e| {
                Struct::new(vec![
                    ("JNL_PlotID", Field::String(e.id.clone())),
                    ("JNL_State", Field::Int(e.stage)),
                    ("JNL_Time", Field::Dword(e.time)),
                    ("JNL_Date", Field::Dword(e.date)),
                ])
            })
            .collect();
        fields.insert("JNL_Entries".to_owned(), Field::List(journal_list));

        let members_list: Vec<_> = pt
            .members
            .iter()
            .map(|m| {
                Struct::new(vec![
                    ("PT_MEMBER_ID", Field::Int(m.idx as i32)),
                    ("PT_IS_LEADER", Field::Byte(m.leader as u8)),
                ])
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
                Struct::new(vec![
                    ("PT_NPC_AVAIL", Field::Byte(m.available as u8)),
                    ("PT_NPC_SELECT", Field::Byte(m.selectable as u8)),
                ])
            })
            .collect();
        fields.insert("PT_AVAIL_NPCS".to_owned(), Field::List(av_members_list));

        fields.insert(
            "PT_CHEAT_USED".to_owned(),
            Field::Byte(self.save.nfo.cheat_used as u8),
        );
        fields.insert("PT_GOLD".to_owned(), Field::Dword(pt.credits));
        fields.insert("PT_XP_POOL".to_owned(), Field::Int(pt.party_xp));

        if let Some(v) = pt.components {
            fields.insert("PT_ITEM_COMPONEN".to_owned(), Field::Dword(v));
        }

        if let Some(v) = pt.chemicals {
            fields.insert("PT_ITEM_CHEMICAL".to_owned(), Field::Dword(v));
        }

        if let Some(v) = &pt.influence {
            let influence_list = v
                .iter()
                .map(|m| Struct::new(vec![("PT_NPC_INFLUENCE", Field::Int(*m))]))
                .collect();
            fields.insert("PT_INFLUENCE".to_owned(), Field::List(influence_list));
        }
    }

    fn update_characters(&mut self) {
        for (idx, char) in self.save.characters.iter().enumerate() {
            Self::update_character(char, &mut self.save.inner.characters[idx]);
        }
    }

    fn update_character(char: &Character, s: &mut Struct) {
        let fields = &mut s.fields;
        fields.insert(
            "FirstName".to_owned(),
            Field::LocString(
                u32::MAX,
                vec![LocString {
                    id: 0,
                    content: char.name.clone(),
                }],
            ),
        );

        let attributes = ["Str", "Dex", "Con", "Int", "Wis", "Cha"];
        for (idx, name) in attributes.into_iter().enumerate() {
            fields.insert(name.to_owned(), Field::Byte(char.attributes[idx]));
        }

        fields.insert("HitPoints".to_owned(), Field::Short(char.hp));
        fields.insert("CurrentHitPoints".to_owned(), Field::Short(char.hp));
        fields.insert("MaxHitPoints".to_owned(), Field::Short(char.hp_max));
        fields.insert("ForcePoints".to_owned(), Field::Short(char.fp));
        fields.insert("CurrentForce".to_owned(), Field::Short(char.fp));
        fields.insert("MaxForcePoints".to_owned(), Field::Short(char.fp_max));
        fields.insert("Min1HP".to_owned(), Field::Byte(char.min_1_hp as u8));
        fields.insert("GoodEvil".to_owned(), Field::Byte(char.good_evil));
        fields.insert("Experience".to_owned(), Field::Dword(char.experience));

        let skills = char
            .skills
            .iter()
            .map(|r| Struct::new(vec![("Rank", Field::Byte(*r))]))
            .collect();
        fields.insert("SkillList".to_owned(), Field::List(skills));

        let feats = char
            .feats
            .iter()
            .map(|f| Struct::with_type(1, vec![("Feat", Field::Word(*f))]))
            .collect();
        fields.insert("FeatList".to_owned(), Field::List(feats));

        let classes = char.classes.iter().map(Self::make_class).collect();
        fields.insert("ClassList".to_owned(), Field::List(classes));

        fields.insert("Gender".to_owned(), Field::Byte(char.gender.to_int()));
        fields.insert("PortraitId".to_owned(), Field::Word(char.portrait));
        fields.insert("Appearance_Type".to_owned(), Field::Word(char.appearance));
        fields.insert("SoundSetFile".to_owned(), Field::Word(char.soundset));
    }

    fn make_class(class: &Class) -> Struct {
        let mut s = Struct::with_type(
            2,
            vec![
                ("ClassLevel", Field::Short(class.level)),
                ("Class", Field::Int(class.id)),
            ],
        );

        if let Some(powers) = &class.powers {
            let powers = powers
                .iter()
                .map(|p| Struct::with_type(3, vec![("Spell", Field::Word(*p))]))
                .collect();

            s.fields
                .insert("KnownList0".to_owned(), Field::List(powers));
            // fuck knows, normal saves have this field
            s.fields.insert(
                "SpellsPerDayList".to_owned(),
                Field::List(vec![Struct::with_type(
                    17767,
                    vec![("NumSpellsLeft", Field::Byte(0))],
                )]),
            );
        }

        s
    }

    fn update_pifo(&mut self) {
        if !self.save.inner.use_pifo {
            return;
        }
        let pifo = self.save.inner.pifo.as_mut().unwrap();
        let Field::List(list) = pifo.content.fields.get_mut("Mod_PlayerList").unwrap() else {
            unreachable!()
        };
        list[0] = self.save.inner.characters[0].clone();
    }

    fn update_erf(&mut self) {
        let erf = &mut self.save.inner.erf;
        let keys: Vec<_> = erf.resources.keys().map(|k| k.0.clone()).collect();
        let map = string_lowercase_map(&keys);

        // pain
        if !self.save.inner.use_pifo {
            let key = &map[&self.save.nfo.last_module.to_lowercase()];
            let module = erf.get_mut(key, ResourceType::Sav).unwrap();
            let mut module_erf = Erf::read(&module.content).unwrap();
            let module_inner = module_erf.get_mut("Module", ResourceType::Ifo).unwrap();
            let mut module_inner_gff = Gff::read(&module_inner.content).unwrap();
            let Field::List(list) = module_inner_gff
                .content
                .fields
                .get_mut("Mod_PlayerList")
                .unwrap()
            else {
                unreachable!()
            };

            list[0] = self.save.inner.characters[0].clone();
            module_inner.content = gff::write(module_inner_gff);
            module.content = erf::write(module_erf.clone());
        }

        for (idx, char) in self.save.inner.characters.iter().enumerate().skip(1) {
            let inner_idx = &self.save.characters[idx].idx.to_string();
            let key = &map[&(NPC_RESOURCE_PREFIX.to_owned() + inner_idx)];
            let Some(res) = erf.get_mut(key, ResourceType::Utc) else {
                unreachable!()
            };
            res.content = gff::write(Gff {
                file_head: ("UTC ", "V3.2").into(),
                content: char.clone(),
            });
        }
    }
}
