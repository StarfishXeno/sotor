use crate::{
    save::{Character, Class},
    ui::{
        styles::{
            set_checkbox_styles, set_combobox_styles, set_selectable_styles, set_slider_styles,
            BLACK, GREEN, GREEN_DARK, RED, WHITE,
        },
        widgets::{color_text, on_hover_text_side, Icon, UiExt},
        UiRef,
    },
    util::{get_data_name, ContextExt},
};
use ahash::HashMap;
use core::{Data, DataDescr, DataSorting, GameDataMapped};
use egui::{
    Color32, ComboBox, CursorIcon, FontSelection, Frame, Grid, Id, Margin, Rounding, Sense,
    WidgetText,
};
use emath::{vec2, Align};
use std::{borrow::Cow, collections::HashSet, fmt::Display, hash::Hash};

pub struct CharAbilities<'a> {
    char: &'a mut Character,
    data: &'a GameDataMapped,
}

impl<'a> CharAbilities<'a> {
    pub fn new(char: &'a mut Character, data: &'a GameDataMapped) -> Self {
        Self { char, data }
    }

    pub fn show(&mut self, ui: UiRef) {
        self.feats(ui);
        ui.separator();
        self.classes(ui);
    }

    fn feats(&mut self, ui: UiRef) {
        let list = &mut self.char.feats;
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

        Self::ability_list(ui, list, &self.data.feats);
    }

    fn classes(&mut self, ui: UiRef) {
        ui.horizontal(|ui| {
            ui.label("Classes: ");
            let class_ids = &mut self.char.classes.iter().map(|c| c.id).collect();
            Self::selection("class", ui, class_ids, &self.data.inner.classes);
            if class_ids.len() > self.char.classes.len() {
                let id = class_ids.pop().unwrap();
                let force_user = self.data.classes.get(&id).unwrap().force_user;
                self.char.classes.push(Class {
                    id,
                    level: 1,
                    powers: force_user.then(Vec::new),
                });
            }
        });

        let mut removed = None;
        for (idx, class) in self.char.classes.iter_mut().enumerate() {
            ui.s_empty();
            Self::class(ui, class, self.data, || removed = Some(idx));
        }
        if let Some(idx) = removed {
            self.char.classes.remove(idx);
        }
    }

    fn ability_list<I: Eq + Hash + Display, E: Data<I> + DataDescr + DataSorting>(
        ui: UiRef,
        list: &mut Vec<I>,
        data: &HashMap<I, E>,
    ) {
        let mut named_list: Vec<_> = list
            .iter()
            .enumerate()
            .map(|(idx, id)| {
                let item = data.get(id);
                let sorting = if let Some(a) = item {
                    Cow::Borrowed(a.get_sorting_name())
                } else {
                    Cow::Owned(format!("UNKNOWN {id}"))
                };
                let description = item.and_then(|i| i.get_description());
                (idx, get_data_name(data, id), description, sorting)
            })
            .collect();
        named_list.sort_unstable_by(|a, b| a.3.cmp(&b.3));

        ui.horizontal_wrapped(|ui| {
            let mut removed = None;
            for (idx, name, description, _) in named_list {
                // complex widgets don't work with _wrapped, so doing it raw
                let padding = vec2(4., 2.);
                let layout_job = WidgetText::RichText(color_text(&name, Color32::PLACEHOLDER))
                    .into_layout_job(ui.style(), FontSelection::Default, Align::Min);
                let galley = ui.fonts(|f| f.layout_job(layout_job));
                let (rect, r) =
                    ui.allocate_at_least(galley.size() + padding + padding, Sense::click());
                let r = r.on_hover_cursor(CursorIcon::PointingHand);
                if let Some(descr) = description {
                    on_hover_text_side(ui, &r, descr);
                }

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

    fn selection<I: Eq + Hash + Copy, E: Data<I> + DataDescr>(
        id: &str,
        ui: UiRef,
        list: &mut Vec<I>,
        data_list: &[E],
    ) {
        let id = Id::new("ec_add_").with(id);

        set_combobox_styles(ui);
        let popup_id = ui.make_persistent_id(Id::new(id)).with("popup");
        let mut added = false;
        ComboBox::from_id_source(id).width(240.).show_ui(ui, |ui| {
            let present: HashSet<_> = list.iter().copied().collect();

            set_selectable_styles(ui);
            let mut selected = None;
            for item in data_list {
                let id = item.get_id();
                if present.contains(id) {
                    continue;
                }
                let r = ui.selectable_value(&mut selected, Some(id), item.get_name());
                if let Some(descr) = item.get_description() {
                    on_hover_text_side(ui, &r, descr);
                }
            }
            if let Some(id) = selected {
                list.push(*id);
                added = true;
            }
        });

        // don't close the box after selecting something
        if added {
            ui.memory_mut(|m| m.open_popup(popup_id));
        }
    }

    fn class(ui: UiRef, class: &mut Class, data: &GameDataMapped, mut remove: impl FnMut()) {
        let id = class.id;
        let show_all = ui.ctx().get_data("ec_powers_all").unwrap_or(false);
        let name = get_data_name(&data.classes, &id);

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
                        let data_list = if show_all {
                            Cow::Borrowed(&data.inner.powers)
                        } else {
                            Cow::Owned(
                                data.inner
                                    .powers
                                    .iter()
                                    .filter(|p| !p.extra)
                                    .cloned()
                                    .collect(),
                            )
                        };

                        ui.horizontal(|ui| {
                            Self::selection(
                                &format!("class_{id}"),
                                ui,
                                list.as_mut().unwrap(),
                                &data_list,
                            );
                            ui.label(color_text("Show all:", GREEN));
                            set_checkbox_styles(ui);
                            let mut checked = show_all;
                            ui.s_checkbox(&mut checked);
                            if checked != show_all {
                                ui.ctx().set_data("ec_powers_all", checked);
                            }
                        });
                    });

                if list.is_none() || list.as_ref().unwrap().is_empty() {
                    return;
                }

                ui.s_empty();
                Self::ability_list(ui, list.as_mut().unwrap(), &data.powers);
            });
    }
}
