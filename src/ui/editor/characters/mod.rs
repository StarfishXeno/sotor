use crate::{
    save::Save,
    ui::{
        styles::{set_combobox_styles, set_selectable_styles, GREEN},
        widgets::{color_text, UiExt},
        UiRef,
    },
    util::ContextExt,
};
use core::GameDataMapped;
use egui::{ComboBox, ScrollArea};

mod abilities;
mod equipment;
mod stats;

use abilities::CharAbilities;
use equipment::CharEquipment;
use stats::CharStats;

pub struct Editor<'a> {
    selected: usize,
    save: &'a mut Save,
    data: &'a GameDataMapped,
}

macro_rules! char {
    ($self:ident) => {
        $self.save.characters[$self.selected]
    };
}

const SELECTED_ID: &str = "ec_selected";

impl<'a> Editor<'a> {
    pub fn new(save: &'a mut Save, data: &'a GameDataMapped) -> Self {
        Self {
            selected: 0,
            save,
            data,
        }
    }

    pub fn show(&mut self, ui: UiRef) {
        self.selected = ui.ctx().get_data(SELECTED_ID).unwrap_or(0);
        ScrollArea::vertical()
            .id_source("ec_scroll")
            .stick_to_bottom(true)
            .show(ui, |ui| {
                ui.horizontal_top(|ui| self.header(ui));
                let char = &mut char!(self);
                ui.separator();
                ui.horizontal_top(|ui| {
                    CharStats::new(char, &mut self.save.party_table, self.data).show(ui);
                });
                ui.s_empty();
                ui.separator();
                CharEquipment::new(char, &mut self.save.inventory, self.data).show(ui);
                ui.s_empty();
                ui.separator();
                CharAbilities::new(char, self.data).show(ui);
            });
    }

    fn header(&mut self, ui: UiRef) {
        ui.vertical(|ui| {
            ui.s_empty();
            ui.label("Character: ");
        });

        set_combobox_styles(ui);
        ComboBox::from_id_source("ec_selection")
            .selected_text(char!(self).get_name())
            .width(120.)
            .show_ui(ui, |ui| {
                set_selectable_styles(ui);
                let mut selected = self.selected;
                for (idx, char) in self.save.characters.iter().enumerate() {
                    ui.selectable_value(&mut selected, idx, char.get_name());
                }
                if selected != self.selected {
                    ui.ctx().set_data(SELECTED_ID, selected);
                }
            });

        ui.vertical(|ui| {
            ui.s_empty();
            ui.label(color_text("Edit name: ", GREEN));
        });
        ui.s_text_edit(&mut char!(self).name, 200.);
    }
}
