use crate::{
    save::{Character, Item},
    ui::{
        styles::{
            set_checkbox_styles, set_radio_styles, set_selectable_styles, set_striped_styles, GREEN,
        },
        widgets::{color_text, on_hover_text_side, UiExt},
        UiRef,
    },
    util::{ColumnCounter, ContextExt},
};
use core::{Data, DataDescr, GameDataMapped, ItemSlot, UsableBy, WeaponType};
use egui::{ComboBox, Grid, Id, Response};
use std::{borrow::Cow, mem};

const BASIC_SLOTS_HUMAN: [(&str, ItemSlot); 7] = [
    ("Implant: ", ItemSlot::Implant),
    ("Head: ", ItemSlot::Head),
    ("Gloves: ", ItemSlot::Gloves),
    ("Arm Left: ", ItemSlot::Arms),
    ("Armor: ", ItemSlot::Armor),
    ("Arm Right: ", ItemSlot::Arms),
    ("Belt: ", ItemSlot::Belt),
];
const BASIC_SLOTS_DROID: [(&str, ItemSlot); 7] = [
    ("Utility Left: ", ItemSlot::Implant),
    ("Sensor: ", ItemSlot::Head),
    ("Utility Right: ", ItemSlot::Implant),
    ("Arm Left: ", ItemSlot::Arms),
    ("Plating: ", ItemSlot::Armor),
    ("Arm Right: ", ItemSlot::Arms),
    ("Shield: ", ItemSlot::Belt),
];
const WEAPON_SLOTS: [[&str; 2]; 2] = [["Mainhand: ", "Offhand: "], ["Mainhand 2: ", "Offhand 2: "]];
const WEAPON_IDX_OFFSET: usize = BASIC_SLOTS_HUMAN.len();

pub struct CharEquipment<'a> {
    char: &'a mut Character,
    inventory: &'a mut Vec<Item>,
    data: &'a GameDataMapped,
    usable_by: UsableBy,
    show_all: bool,
}

impl<'a> CharEquipment<'a> {
    pub fn new(
        char: &'a mut Character,
        inventory: &'a mut Vec<Item>,
        data: &'a GameDataMapped,
    ) -> Self {
        Self {
            char,
            inventory,
            data,

            usable_by: UsableBy::Humans,
            show_all: false,
        }
    }

    pub fn show(&mut self, ui: UiRef) {
        self.header(ui);
        self.equipment(ui);
    }

    fn header(&mut self, ui: UiRef) {
        let usable_by = ui
            .ctx()
            .get_data("ec_eq_usable")
            .unwrap_or_else(|| Self::droid_or_human(&self.char.tag));
        let show_all = ui.ctx().get_data("ec_eq_all").unwrap_or(false);
        self.usable_by = usable_by;
        self.show_all = show_all;

        ui.horizontal(|ui| {
            set_radio_styles(ui);
            ui.radio_value(&mut self.usable_by, UsableBy::Humans, "Human gear");
            ui.radio_value(&mut self.usable_by, UsableBy::Droids, "Droid gear");

            set_checkbox_styles(ui);
            ui.s_checkbox(&mut self.show_all);
            ui.label(color_text("Show all", GREEN));
        });

        if usable_by != self.usable_by {
            ui.ctx().set_data("ec_eq_usable", self.usable_by);
        }
        if show_all != self.show_all {
            ui.ctx().set_data("ec_eq_all", self.show_all);
        }
    }

    fn equipment(&mut self, ui: UiRef) {
        let basic_slots = if self.usable_by == UsableBy::Humans {
            BASIC_SLOTS_HUMAN
        } else {
            BASIC_SLOTS_DROID
        };

        set_striped_styles(ui);
        Grid::new(ui.next_auto_id())
            .spacing([10., 6.])
            .striped(true)
            .show(ui, |ui| {
                let mut counter = ColumnCounter::new(2);
                set_checkbox_styles(ui);
                for (idx, (slot_name, slot)) in basic_slots.into_iter().enumerate() {
                    self.slot(ui, slot_name, idx, &[slot]);
                    counter.next(ui);
                }
                ui.end_row();

                self.weapon_slots(ui);
            });
    }

    fn weapon_slots(&mut self, ui: UiRef) {
        let mut counter = ColumnCounter::new(2);
        let all_weapon_item_slots = WeaponType::LIST.map(ItemSlot::Weapon);
        for (gidx, group) in WEAPON_SLOTS.into_iter().enumerate() {
            let mh_idx = WEAPON_IDX_OFFSET + gidx * 2;
            let mh = &self.char.equipment[mh_idx];
            let base_item = mh
                .as_ref()
                .and_then(|i| self.data.inner.base_items.get(&i.base_item));
            for (sidx, slot_name) in group.into_iter().enumerate() {
                let idx = mh_idx + sidx;
                // can equip anything in mh
                let slots = (idx != mh_idx)
                    .then_some(base_item)
                    .flatten()
                    .and_then(|i| {
                        if let ItemSlot::Weapon(tp) = i.slot {
                            return Some(tp.offhand_option());
                        }
                        // this shouldn't happen
                        None
                    })
                    .unwrap_or(&all_weapon_item_slots);

                self.slot(ui, slot_name, idx, slots);
                counter.next(ui);
            }
        }
    }

    fn slot(&mut self, ui: UiRef, slot_name: &str, idx: usize, slots: &[ItemSlot]) {
        let id = Id::new("ec_eq").with(idx);
        ui.label(color_text(slot_name, GREEN));

        ui.add_enabled_ui(!slots.is_empty(), |ui| {
            let current = &mut self.char.equipment[idx];
            let current_id = current.as_ref().map(|i| i.tag.clone());
            let r = ComboBox::from_id_source(id)
                .selected_text(current.as_ref().map(Item::get_name).unwrap_or_default())
                .width(280.)
                .show_ui(ui, |ui| {
                    set_selectable_styles(ui);
                    let mut selected = current_id.as_deref();
                    for template in &self.data.inner.items {
                        let Some(base) = self.data.inner.base_items.get(&template.base_item) else {
                            continue;
                        };
                        if !slots.iter().any(|s| *s == base.slot) {
                            continue;
                        }
                        if base.usable_by != UsableBy::All && base.usable_by != self.usable_by {
                            continue;
                        }
                        if !self.show_all && template.name.is_none() {
                            continue;
                        }

                        let r = ui.selectable_value(
                            &mut selected,
                            Some(template.id.as_str()),
                            template.get_name(),
                        );
                        Self::item_on_hover_text(ui, &r, template);
                    }
                    if selected != current_id.as_deref() {
                        if let Some(id) = selected {
                            let mut new_item: Item = self.data.items.get(id).unwrap().into();
                            if let Some(previous) = current.as_mut() {
                                mem::swap(&mut new_item, previous);
                                self.inventory.push(new_item);
                            } else {
                                *current = Some(new_item);
                            }
                        } else {
                            *current = None;
                        }
                    }
                });

            if let Some(item) = current {
                Self::item_on_hover_text(ui, &r.response, item);
            }
        });
    }

    fn droid_or_human(tag: &str) -> UsableBy {
        match tag {
            "t3m4" | "hk47" | "g0t0" | "remote" => UsableBy::Droids,
            _ => UsableBy::Humans,
        }
    }

    fn item_on_hover_text(ui: UiRef, r: &Response, item: &(impl DataDescr + Data<String>)) {
        let mut hover_text = Cow::Borrowed(item.get_id().as_str());
        if let Some(descr) = item.get_description() {
            hover_text = Cow::Owned([&hover_text, descr].join("\n\n"));
        }
        on_hover_text_side(ui, r, &hover_text);
    }
}
