use std::collections::HashSet;

use crate::{
    save::{Character, PartyTable, Save},
    ui::{
        styles::{
            set_checkbox_styles, set_combobox_styles, set_drag_value_styles, set_selectable_styles,
            set_striped_styles, BLACK, GREEN, GREEN_DARK, RED, WHITE,
        },
        widgets::{color_text, Icon, IconButton, UiExt},
        UiRef,
    },
    util::ContextExt,
};
use ahash::HashMap;
use egui::{
    Color32, ComboBox, CursorIcon, DragValue, FontSelection, Grid, Rounding, ScrollArea, Sense,
    WidgetText,
};
use emath::{vec2, Align};
use internal::{Feat, GameDataMapped};

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

        ScrollArea::vertical()
            .id_source("ec_scroll")
            .stick_to_bottom(true)
            .show(ui, |ui| {
                ui.horizontal(|ui| self.header(ui));
                ui.separator();
                ui.s_offset(0., 5.);
                ui.horizontal_top(|ui| self.main_stats(ui));
                ui.s_offset(0., 5.);
                ui.separator();
                self.feats(ui);
                ui.separator();
            });
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

        Self::grid("ec_general", ui, |ui| {
            ui.label(color_text("Tag: ", GREEN));
            ui.s_text(if char.tag.is_empty() {
                "None"
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
            for (name, value) in labels.into_iter().zip(&mut char.skills) {
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
            for (name, value) in labels.into_iter().zip(&mut char.attributes) {
                ui.label(color_text(name, GREEN));
                ui.add(DragValue::new(value));
                ui.end_row();
            }
        });
    }

    fn grid(id: &str, ui: UiRef, add_contents: impl FnOnce(UiRef)) {
        Grid::new(id)
            .striped(true)
            .spacing([20., 6.])
            .show(ui, add_contents);
    }

    fn feats(&mut self, ui: UiRef) {
        ui.label("Feats:");
        let list = &mut char!(self).feats;
        let data = &self.data.feats;
        Self::feat_list(ui, list, data);
        ui.s_empty();
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.s_empty();
                ui.label("Add feat: ");
            });
            Self::feat_selection(ui, list, data, &self.data.inner.feats);
        });
    }
    fn feat_list(ui: UiRef, list: &mut Vec<u16>, data: &HashMap<u16, Feat>) {
        let mut named_list: Vec<_> = list
            .iter()
            .enumerate()
            .map(|(idx, id)| {
                let (name, sorting_name) = if let Some(feat) = data.get(id) {
                    (feat.name.clone(), feat.sorting_name.clone())
                } else {
                    let name = format!("UNKNOWN {id}");
                    (name.clone(), name)
                };
                (idx, name, sorting_name)
            })
            .collect();
        named_list.sort_unstable_by(|a, b| a.2.cmp(&b.2));

        ui.horizontal_wrapped(|ui| {
            let mut removed = None;
            for (idx, name, _) in named_list {
                // complex widgets don't work with _wrapped, so doing it raw
                let padding = vec2(4., 2.);
                let layout_job = WidgetText::RichText(color_text(&name, Color32::PLACEHOLDER))
                    .into_layout_job(ui.style(), FontSelection::Default, Align::Min);
                let galley = ui.fonts(|f| f.layout_job(layout_job));
                let (rect, r) =
                    ui.allocate_at_least(galley.size() + padding + padding, Sense::click());
                let r = r.on_hover_cursor(CursorIcon::PointingHand);

                let (background, text) = if r.clicked() || r.is_pointer_button_down_on() {
                    (RED, WHITE)
                } else if r.hovered() {
                    (WHITE, BLACK)
                } else {
                    (GREEN_DARK, WHITE)
                };

                ui.painter()
                    .rect(rect, Rounding::same(2.), background, (0., GREEN_DARK));
                let pos = ui
                    .layout()
                    .align_size_within_rect(galley.size(), rect.shrink2(padding))
                    .min;
                ui.painter().galley(pos, galley, text);

                if r.clicked() {
                    removed = Some(idx);
                }
            }
            if let Some(idx) = removed {
                list.remove(idx);
            }
        });
    }

    fn feat_selection(
        ui: UiRef,
        list: &mut Vec<u16>,
        data: &HashMap<u16, Feat>,
        data_list: &[Feat],
    ) {
        let present: HashSet<_> = list.iter().copied().collect();
        let available: Vec<_> = data_list
            .iter()
            .filter(|f| !present.contains(&f.id))
            .collect();
        let selected = ui.ctx().get_data("eq_add_feat").flatten();
        let selected_name = selected
            .and_then(|id| data.get(&id))
            .map_or("", |f| &f.name);
        set_combobox_styles(ui);
        ComboBox::from_id_source("ec_add_feat")
            .width(240.)
            .selected_text(selected_name)
            .show_ui(ui, |ui| {
                set_selectable_styles(ui);
                let mut selected_tmp = selected;
                for feat in available {
                    ui.selectable_value(&mut selected_tmp, Some(feat.id), &feat.name);
                }
                if selected_tmp != selected {
                    ui.ctx().set_data("eq_add_feat", selected_tmp);
                }
            });

        let btn = ui.add_enabled(selected.is_some(), IconButton::new(Icon::Plus));
        if btn.clicked() {
            list.push(selected.unwrap());
            ui.ctx().set_data::<Option<u16>>("eq_add_feat", None);
        }
    }
}
