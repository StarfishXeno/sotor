use crate::{
    save::{Character, PartyTable, Save},
    ui::{
        styles::{
            set_checkbox_styles, set_combobox_styles, set_drag_value_styles, set_selectable_styles,
            set_striped_styles, GREEN,
        },
        widgets::{color_text, UiExt},
        UiRef,
    },
    util::ContextExt,
};
use egui::{ComboBox, DragValue, Grid, Id};
use internal::GameDataMapped;

pub struct Editor<'a> {
    selected: usize,
    party_table: &'a mut PartyTable,
    characters: &'a mut Vec<Character>,
    data: &'a GameDataMapped,
}

macro_rules! char {
    ($self:ident) => {
        &mut $self.characters[$self.selected]
    };
}
const SELECTED_ID: &str = "ec_selected";

impl<'a> Editor<'a> {
    pub fn new(save: &'a mut Save, data: &'a GameDataMapped) -> Self {
        Self {
            selected: 0,
            party_table: &mut save.party_table,
            characters: &mut save.characters,
            data,
        }
    }

    pub fn show(&mut self, ui: UiRef) {
        self.selected = ui.ctx().get_data(SELECTED_ID).unwrap_or(0);

        ui.horizontal(|ui| self.header(ui));
        ui.separator();
        ui.s_offset(0., 5.);
        ui.horizontal_top(|ui| self.main_stats(ui));
    }

    fn header(&mut self, ui: UiRef) {
        ui.vertical(|ui| {
            ui.s_empty();
            ui.label("Character: ");
        });

        set_combobox_styles(ui);
        ComboBox::from_id_source("ec_selection")
            .selected_text(self.characters[self.selected].get_name())
            .show_ui(ui, |ui| {
                set_selectable_styles(ui);
                let mut selected = self.selected;
                for (idx, char) in self.characters.iter().enumerate() {
                    ui.selectable_value(&mut selected, idx, char.get_name());
                }
                if selected != self.selected {
                    ui.ctx().set_data(SELECTED_ID, selected);
                }
            });

        ui.vertical(|ui| {
            ui.s_offset(0., 0.);
            ui.label(color_text("Edit name: ", GREEN));
        });
        ui.s_text_edit(&mut char!(self).name, 200.);
    }

    fn main_stats(&mut self, ui: UiRef) {
        let char = char!(self);
        set_striped_styles(ui);
        set_drag_value_styles(ui);

        Self::grid("ec_general".into(), ui, |ui| {
            ui.label(color_text("Tag: ", GREEN));
            ui.s_text(if char.tag.is_empty() {
                "Player character"
            } else {
                &char.tag
            });
            ui.end_row();

            ui.label(color_text("Max HP: ", GREEN));
            ui.s_slider(&mut char.hp_max, 0..=9999, true);
            ui.end_row();

            ui.label(color_text("HP: ", GREEN));
            ui.s_slider(&mut char.hp, 0..=char.hp_max, false);
            ui.end_row();

            ui.label(color_text("Min 1 HP: ", GREEN));
            ui.horizontal(|ui| {
                set_checkbox_styles(ui);
                ui.s_checkbox(&mut char.min_1_hp);
            });
            ui.end_row();

            ui.label(color_text("Max FP: ", GREEN));
            ui.s_slider(&mut char.fp_max, 0..=9999, true);
            ui.end_row();

            ui.label(color_text("FP: ", GREEN));
            ui.s_slider(&mut char.fp, 0..=char.fp_max, false);
            ui.end_row();

            ui.label(color_text("Alignment: ", GREEN));
            ui.s_slider(&mut char.good_evil, 0..=100, false);
            ui.end_row();

            ui.label(color_text("XP: ", GREEN));
            ui.s_slider(&mut char.experience, 0..=9_999_999, true);
            ui.end_row();

            if let Some(list) = &mut self.party_table.influence {
                if let Some(inf) = list.get_mut(char.idx) {
                    ui.label(color_text("Influence: ", GREEN));
                    ui.s_slider(inf, 0..=100, false);
                    ui.end_row();
                }
            }
        });

        Self::grid("ec_skills".into(), ui, |ui| {
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
            for (name, value) in labels.into_iter().zip(&mut char.skills) {
                ui.label(color_text(name, GREEN));
                ui.add(DragValue::new(value));
                ui.end_row();
            }
        });

        Self::grid("ec_attributes".into(), ui, |ui| {
            let labels = [
                "Strength: ",
                "Dexterity: ",
                "Constitution: ",
                "Intelligence: ",
                "Wisdom: ",
                "Charisma: ",
            ];
            for (name, value) in labels.into_iter().zip(&mut char.attributes) {
                ui.label(color_text(name, GREEN));
                ui.add(DragValue::new(value));
                ui.end_row();
            }
        });
    }

    fn grid(id: Id, ui: UiRef, add_contents: impl FnOnce(UiRef)) {
        Grid::new(id)
            .striped(true)
            .spacing([20., 6.])
            .show(ui, add_contents);
    }
}
