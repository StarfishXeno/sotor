use crate::{
    save::{Character, Gender, PartyTable},
    ui::{
        styles::{
            set_checkbox_styles, set_combobox_styles, set_drag_value_styles, set_selectable_styles,
            set_striped_styles, GREEN,
        },
        widgets::{color_text, UiExt},
        UiRef,
    },
    util::get_data_name,
};
use ahash::HashMap;
use core::{Appearance, GameDataMapped};
use egui::{ComboBox, DragValue, Grid};

pub struct CharStats<'a> {
    char: &'a mut Character,
    party_table: &'a mut PartyTable,
    data: &'a GameDataMapped,
}

impl<'a> CharStats<'a> {
    pub fn new(
        char: &'a mut Character,
        party_table: &'a mut PartyTable,
        data: &'a GameDataMapped,
    ) -> Self {
        Self {
            char,
            party_table,
            data,
        }
    }

    pub fn show(&mut self, ui: UiRef) {
        set_striped_styles(ui);
        set_drag_value_styles(ui);

        Self::grid("ec_general", ui, |ui| {
            ui.label(color_text("Tag: ", GREEN));
            ui.s_text(if self.char.tag.is_empty() {
                "None"
            } else {
                &self.char.tag
            });
            ui.end_row();

            ui.label(color_text("Max HP: ", GREEN));
            ui.s_slider(&mut self.char.hp_max, 0..=9999, true);
            ui.end_row();

            ui.label(color_text("HP: ", GREEN));
            ui.s_slider(&mut self.char.hp, 0..=self.char.hp_max, false);
            ui.end_row();

            ui.label(color_text("Min 1 HP: ", GREEN));
            ui.horizontal(|ui| {
                set_checkbox_styles(ui);
                ui.s_checkbox(&mut self.char.min_1_hp);
            });
            ui.end_row();

            ui.label(color_text("Max FP: ", GREEN));
            ui.s_slider(&mut self.char.fp_max, 0..=9999, true);
            ui.end_row();

            ui.label(color_text("FP: ", GREEN));
            ui.s_slider(&mut self.char.fp, 0..=self.char.fp_max, false);
            ui.end_row();

            ui.label(color_text("Alignment: ", GREEN));
            ui.s_slider(&mut self.char.good_evil, 0..=100, false);
            ui.end_row();

            ui.label(color_text("XP: ", GREEN));
            ui.s_slider(&mut self.char.experience, 0..=9_999_999, true);
            ui.end_row();

            if let Some(list) = &mut self.party_table.influence {
                if let Some(inf) = list.get_mut(self.char.idx) {
                    ui.label(color_text("Influence: ", GREEN));
                    ui.s_slider(inf, 0..=100, false);
                    ui.end_row();
                }
            }
        });

        Self::grid("ec_skills", ui, |ui| {
            let labels = [
                "Computer Use: ",
                "Demolitions: ",
                "Stealth: ",
                "Awareness: ",
                "Persuade: ",
                "Repair: ",
                "Security: ",
                "Treat Injury: ",
            ];
            for (name, value) in labels.into_iter().zip(&mut self.char.skills) {
                ui.label(color_text(name, GREEN));
                ui.add(DragValue::new(value));
                ui.end_row();
            }
        });

        Self::grid("ec_attributes", ui, |ui| {
            let labels = [
                "Strength: ",
                "Dexterity: ",
                "Constitution: ",
                "Intelligence: ",
                "Wisdom: ",
                "Charisma: ",
            ];
            for (name, value) in labels.into_iter().zip(&mut self.char.attributes) {
                ui.label(color_text(name, GREEN));
                ui.add(DragValue::new(value));
                ui.end_row();
            }
        });
        self.appearance(ui);
    }

    fn grid(id: &str, ui: UiRef, add_contents: impl FnOnce(UiRef)) {
        Grid::new(id)
            .striped(true)
            .spacing([10., 6.])
            .show(ui, add_contents);
    }

    fn appearance(&mut self, ui: UiRef) {
        set_striped_styles(ui);
        Self::grid("ec_appearance", ui, |ui| {
            set_combobox_styles(ui);
            ui.label(color_text("Gender:", GREEN));
            ui.end_row();
            ComboBox::from_id_source("ec_gender")
                .width(200.)
                .selected_text(self.char.gender.to_str())
                .show_ui(ui, |ui| {
                    set_selectable_styles(ui);
                    let mut selected = self.char.gender;
                    for gender in Gender::LIST {
                        ui.selectable_value(&mut selected, gender, gender.to_str());
                    }
                    self.char.gender = selected;
                });
            ui.end_row();
            ui.label(color_text("Portrait:", GREEN));
            ui.end_row();
            Self::appearance_selection(
                ui,
                "ec_portrait",
                &mut self.char.portrait,
                &self.data.portraits,
                &self.data.inner.portraits,
            );
            ui.end_row();
            ui.label(color_text("Appearance:", GREEN));
            ui.end_row();
            Self::appearance_selection(
                ui,
                "ec_appearance",
                &mut self.char.appearance,
                &self.data.appearances,
                &self.data.inner.appearances,
            );
            ui.end_row();
            ui.label(color_text("Soundset:", GREEN));
            ui.end_row();
            Self::appearance_selection(
                ui,
                "ec_soundset",
                &mut self.char.soundset,
                &self.data.soundsets,
                &self.data.inner.soundsets,
            );
        });
    }

    fn appearance_selection(
        ui: UiRef,
        id: &str,
        current: &mut u16,
        data: &HashMap<u16, Appearance>,
        data_list: &[Appearance],
    ) {
        let name = get_data_name(data, current);

        ComboBox::from_id_source(id)
            .width(200.)
            .selected_text(name)
            .show_ui(ui, |ui| {
                set_selectable_styles(ui);
                let mut selected = *current;
                for item in data_list {
                    ui.selectable_value(&mut selected, item.id, &item.name);
                }
                *current = selected;
            });
    }
}
