use crate::{
    save::{Character, Class, PartyTable, Save},
    ui::{
        styles::{
            set_checkbox_styles, set_combobox_styles, set_drag_value_styles, set_selectable_styles,
            set_slider_styles, set_striped_styles, BLACK, GREEN, GREEN_DARK, RED, WHITE,
        },
        widgets::{color_text, Icon, UiExt},
        UiRef,
    },
    util::ContextExt,
};
use ahash::HashMap;
use egui::{
    Color32, ComboBox, CursorIcon, DragValue, FontSelection, Frame, Grid, Id, Margin, Rounding,
    ScrollArea, Sense, WidgetText,
};
use emath::{vec2, Align};
use internal::{Appearance, Data, Feat, GameDataMapped};
use std::collections::HashSet;
use std::hash::Hash;

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
                ui.horizontal_top(|ui| self.header(ui));
                ui.separator();
                ui.horizontal_top(|ui| self.main_stats(ui));
                ui.s_empty();
                ui.separator();
                ui.horizontal_top(|ui| self.appearance(ui));
                ui.separator();
                self.feats(ui);
                ui.separator();
                self.classes(ui);
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
            .width(120.)
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

    fn appearance(&mut self, ui: UiRef) {
        let char = char!(self);
        set_striped_styles(ui);
        Self::grid("ec_appearance", ui, |ui| {
            ui.label("Portrait:");
            ui.label("Appearance:");
            ui.label("Soundset:");
            ui.end_row();

            set_combobox_styles(ui);
            Self::appearance_selection(
                ui,
                "ec_portrait",
                &mut char.portrait,
                &self.data.portraits,
                &self.data.inner.portraits,
            );
            Self::appearance_selection(
                ui,
                "ec_appearance",
                &mut char.appearance,
                &self.data.appearances,
                &self.data.inner.appearances,
            );
            Self::appearance_selection(
                ui,
                "ec_soundset",
                &mut char.soundset,
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
        let name = data
            .get(current)
            .map_or_else(|| format!("UNKNOWN {current}"), |a| a.name.clone());
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

    fn feats(&mut self, ui: UiRef) {
        let list = &mut char!(self).feats;
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.s_empty();
                ui.label("Feats: ");
            });
            Self::selection("feats", ui, list, &self.data.inner.feats);
        });
        if list.is_empty() {
            return;
        }
        ui.s_empty();
        Self::feat_list(ui, list, &self.data.feats);
    }

    fn classes(&mut self, ui: UiRef) {
        let char = char!(self);
        ui.horizontal(|ui| {
            ui.label("Classes: ");
            let class_ids = &mut char.classes.iter().map(|c| c.id).collect();
            Self::selection("class", ui, class_ids, &self.data.inner.classes);
            if class_ids.len() > char.classes.len() {
                let id = class_ids.pop().unwrap();
                let force_user = self.data.classes.get(&id).unwrap().force_user;
                char.classes.push(Class {
                    id,
                    level: 1,
                    powers: force_user.then(Vec::new),
                });
            }
        });

        let mut removed = None;
        for (idx, class) in char.classes.iter_mut().enumerate() {
            ui.s_empty();
            Self::class(ui, class, self.data, || removed = Some(idx));
        }
        if let Some(idx) = removed {
            char.classes.remove(idx);
        }
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

    fn selection<I: Copy + Eq + Hash, E: Data<I>>(
        id: &str,
        ui: UiRef,
        list: &mut Vec<I>,
        data_list: &[E],
    ) {
        let id = Id::new("ec_add_").with(id);
        let present: HashSet<_> = list.iter().copied().collect();
        let available: Vec<_> = data_list
            .iter()
            .filter(|f| !present.contains(&f.get_id()))
            .collect();

        set_combobox_styles(ui);
        let popup_id = ui.make_persistent_id(id).with("popup");
        let mut added = false;
        ComboBox::from_id_source(id).width(240.).show_ui(ui, |ui| {
            set_selectable_styles(ui);
            let mut selected = None;
            for item in available {
                ui.selectable_value(&mut selected, Some(item.get_id()), item.get_name());
            }
            if let Some(id) = selected {
                list.push(id);
                added = true;
            }
        });

        // don't close the box after adding a feat
        if added {
            ui.memory_mut(|m| m.open_popup(popup_id));
        }
    }

    fn class(ui: UiRef, class: &mut Class, data: &GameDataMapped, mut remove: impl FnMut()) {
        let id = class.id;
        let name = data
            .classes
            .get(&class.id)
            .map_or_else(|| format!("UNKNOWN {id}"), |c| c.name.clone());

        Frame::default()
            .rounding(2.)
            .stroke((2., GREEN_DARK))
            .outer_margin(Margin::ZERO)
            .inner_margin(Margin::same(6.))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    if ui.s_icon_button(Icon::Remove, "Remove class").clicked() {
                        remove();
                    }
                    ui.s_text(&name);
                });
                let list = &mut class.powers;

                Grid::new(format!("ec_class_{id}"))
                    .spacing([20., 6.])
                    .show(ui, |ui| {
                        ui.label("Level: ");
                        set_slider_styles(ui);
                        ui.s_slider(&mut class.level, 0..=40, false);
                        ui.end_row();
                        if list.is_none() {
                            return;
                        }
                        ui.label(color_text("Powers: ", GREEN));
                        Self::selection(
                            &format!("class_{id}"),
                            ui,
                            list.as_mut().unwrap(),
                            &data.inner.powers,
                        );
                    });

                if list.is_none() || list.as_ref().unwrap().is_empty() {
                    return;
                }

                ui.s_empty();
                Self::feat_list(ui, list.as_mut().unwrap(), &data.powers);
            });
    }
}
