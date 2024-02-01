use crate::{
    save::{Character, Class, GlobalValue, Item, Save, GLOBALS_TYPES, NPC_RESOURCE_PREFIX},
    util::calc_hp_fp_offset,
};
use core::{
    erf::{self, Erf},
    gff::{self, Field, Gff, Struct},
    GameDataMapped, LocString, ReadResourceNoArg as _, ResourceType,
};

use super::EQUIPMENT_SLOT_IDS;

pub struct Updater<'a> {
    save: &'a mut Save,
    data: &'a GameDataMapped,
}

impl<'a> Updater<'a> {
    pub fn new(save: &'a mut Save, data: &'a GameDataMapped) -> Self {
        Self { save, data }
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
        let s = &mut self.save.inner.nfo.content;

        s.insert("SAVEGAMENAME", Field::String(nfo.save_name.clone()));
        s.insert("CHEATUSED", Field::Byte(nfo.cheat_used as u8));
        // PC data can be temporarily overriden by an NPC, this preserves the name in that case
        if self.save.characters[0].tag.is_empty() {
            s.insert(
                "PCNAME",
                Field::String(self.save.characters.first().unwrap().name.clone()),
            );
        }

        let mut char_indices = vec![usize::MAX]; // PC
        for member in &self.save.party_table.members {
            char_indices.push(member.idx);
        }
        for (idx, char_idx) in char_indices.into_iter().enumerate() {
            let Some(char) = self.save.characters.iter().find(|c| c.idx == char_idx) else {
                continue;
            };
            let Some(portrait) = self.data.portraits.get(&char.portrait) else {
                continue;
            };
            s.fields.insert(
                format!("PORTRAIT{idx}"),
                Field::ResRef(portrait.name.clone()),
            );
        }
    }

    fn update_globals(&mut self) {
        let globals = &self.save.globals;
        let s = &mut self.save.inner.globals.content;
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
            s.fields.insert(format!("Cat{name}"), Field::List(structs));
        }

        // NUMBERS
        let number_values = numbers.iter().map(|g| g.1).collect();
        s.insert("ValNumber", Field::Void(number_values));

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
        s.insert("ValBoolean", Field::Void(boolean_values));
    }

    fn update_party_table(&mut self) {
        let s = &mut self.save.inner.party_table.content;
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
        s.insert("JNL_Entries", Field::List(journal_list));

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
        s.insert("PT_NUM_MEMBERS", Field::Byte(members_list.len() as u8));
        s.insert("PT_MEMBERS", Field::List(members_list));

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
        s.insert("PT_AVAIL_NPCS", Field::List(av_members_list));

        s.insert("PT_CHEAT_USED", Field::Byte(self.save.nfo.cheat_used as u8));
        s.insert("PT_GOLD", Field::Dword(pt.credits));
        s.insert("PT_XP_POOL", Field::Int(pt.party_xp));

        if let Some(v) = pt.components {
            s.insert("PT_ITEM_COMPONEN", Field::Dword(v));
        }
        if let Some(v) = pt.chemicals {
            s.insert("PT_ITEM_CHEMICAL", Field::Dword(v));
        }
        if let Some(v) = &pt.influence {
            let influence_list = v
                .iter()
                .map(|m| Struct::new(vec![("PT_NPC_INFLUENCE", Field::Int(*m))]))
                .collect();
            s.insert("PT_INFLUENCE", Field::List(influence_list));
        }
    }

    fn update_characters(&mut self) {
        for (idx, char) in self.save.characters.iter().enumerate() {
            Self::update_character(char, &mut self.save.inner.characters[idx]);
        }
    }

    fn update_character(char: &Character, s: &mut Struct) {
        s.insert(
            "FirstName",
            Field::LocString((
                char.name_ref,
                vec![LocString {
                    id: 0,
                    content: char.name.clone(),
                }],
            )),
        );

        let attributes = ["Str", "Dex", "Con", "Int", "Wis", "Cha"];
        for (idx, name) in attributes.into_iter().enumerate() {
            s.insert(name, Field::Byte(char.attributes[idx]));
        }

        let (hp, fp) = calc_hp_fp_offset(char);
        let current_health = char.hp - hp;
        let current_force = char.fp - fp;

        s.insert("HitPoints", Field::Short(current_health));
        s.insert("CurrentHitPoints", Field::Short(current_health));
        s.insert("MaxHitPoints", Field::Short(char.hp_max));
        s.insert("ForcePoints", Field::Short(current_force));
        s.insert("CurrentForce", Field::Short(current_force));
        s.insert("MaxForcePoints", Field::Short(char.fp_max));
        s.insert("Min1HP", Field::Byte(char.min_1_hp as u8));
        s.insert("GoodEvil", Field::Byte(char.good_evil));
        s.insert("Experience", Field::Dword(char.experience));

        let skills = char
            .skills
            .iter()
            .map(|r| Struct::new(vec![("Rank", Field::Byte(*r))]))
            .collect();
        s.insert("SkillList", Field::List(skills));

        let feats = char
            .feats
            .iter()
            .map(|f| Struct::with_type(1, vec![("Feat", Field::Word(*f))]))
            .collect();
        s.insert("FeatList", Field::List(feats));

        let classes = char.classes.iter().map(Self::make_class).collect();
        s.insert("ClassList", Field::List(classes));

        s.insert("Gender", Field::Byte(char.gender.to_int()));
        s.insert("PortraitId", Field::Word(char.portrait));
        s.insert("Appearance_Type", Field::Word(char.appearance));
        s.insert("SoundSetFile", Field::Word(char.soundset));

        let mut equipment = Vec::with_capacity(EQUIPMENT_SLOT_IDS.len());
        for (idx, tp) in EQUIPMENT_SLOT_IDS.into_iter().enumerate() {
            let Some(item) = &char.equipment[idx] else {
                continue;
            };
            let mut item_s = Self::make_item(item);
            item_s.tp = tp;

            equipment.push(item_s);
        }
        s.insert("Equip_ItemList", Field::List(equipment));
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

            s.insert("KnownList0", Field::List(powers));
            // fuck knows, normal saves have this field
            s.insert(
                "SpellsPerDayList",
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
        let inventory = self.make_inventory();
        let erf = &mut self.save.inner.erf;

        let inventory_res = erf.get_mut("inventory", ResourceType::Unknown).unwrap();
        inventory_res.content = gff::write(inventory);

        // pain
        if !self.save.inner.use_pifo {
            let module = erf
                .get_mut(&self.save.nfo.last_module.to_lowercase(), ResourceType::Sav)
                .unwrap();
            let mut module_erf = Erf::read(&module.content).unwrap();
            let module_inner = module_erf.get_mut("module", ResourceType::Ifo).unwrap();
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
            let inner_idx = self.save.characters[idx].idx;
            let key = if inner_idx == usize::MAX - 1 {
                "pc".to_owned()
            } else {
                format!("{NPC_RESOURCE_PREFIX}{inner_idx}")
            };
            let res = erf.get_mut(&key, ResourceType::Utc).unwrap();
            res.content = gff::write(Gff {
                file_head: ("UTC ", "V3.2").into(),
                content: char.clone(),
            });
        }
    }

    fn make_item(item: &Item) -> Struct {
        let mut s = Struct {
            tp: 0,
            fields: item.raw.clone(),
        };
        s.insert("StackSize", Field::Word(item.stack_size));
        s.insert("MaxCharges", Field::Byte(item.max_charges));
        s.insert("Charges", Field::Byte(item.charges));
        s.insert("NewItem", Field::Byte(item.new as u8));
        s.insert("Upgrades", Field::Dword(item.upgrades));
        if let Some(slots) = item.upgrade_slots {
            for (idx, v) in slots.into_iter().enumerate() {
                s.fields.insert(format!("UpgradeSlot{idx}"), Field::Int(v));
            }
        }
        s.insert("NewItem", Field::Byte(item.new as u8));
        let mut insert_loc_if_needed = |source: Option<&String>, key: &str| {
            let Some(val) = source else {
                return;
            };
            let (str_ref, v) = s.fields[key].loc_string_unwrap();
            if !v.is_empty() {
                return;
            }
            let value = Field::LocString((
                *str_ref,
                vec![LocString {
                    id: 0,
                    content: val.clone(),
                }],
            ));
            s.insert(key, value);
        };
        insert_loc_if_needed(item.name.as_ref(), "LocalizedName");
        insert_loc_if_needed(item.description.as_ref(), "DescIdentified");

        s
    }

    fn make_inventory(&mut self) -> Gff {
        let mut items = Vec::with_capacity(self.save.inventory.len());
        for item in &self.save.inventory {
            items.push(Self::make_item(item));
        }

        Gff {
            file_head: ("INV ", "V3.2").into(),
            content: Struct::with_type(u32::MAX, vec![("ItemList", Field::List(items))]),
        }
    }
}
