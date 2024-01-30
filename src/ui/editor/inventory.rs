use crate::{
    save::{Item, Save},
    ui::{
        styles::{
            set_checkbox_styles, set_combobox_styles, set_selectable_styles, set_slider_styles,
            set_striped_styles, GREEN, WHITE,
        },
        widgets::{color_text, on_hover_text_side, Icon, UiExt},
        UiRef,
    },
    util::ContextExt,
};
use core::{Data as _, GameDataMapped};
use egui::{ComboBox, Grid, Label, ScrollArea, Separator};
use emath::{vec2, Align};

pub struct Editor<'a> {
    items: &'a mut Vec<Item>,
    data: &'a GameDataMapped,
    selected: usize,
}

impl<'a> Editor<'a> {
    pub fn new(save: &'a mut Save, data: &'a GameDataMapped) -> Self {
        Self {
            items: &mut save.inventory,
            data,
            selected: 0,
        }
    }

    pub fn show(&mut self, ui: UiRef) {
        ui.horizontal_top(|ui| {
            let spacing = &mut ui.spacing_mut().item_spacing;
            let prev_spacing = *spacing;
            *spacing = vec2(0., 4.);
            ScrollArea::vertical()
                .id_source("ei_scroll")
                .max_width(220.)
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        self.list(ui);
                    });
                });
            ui.add(Separator::default().spacing(1.));
            ui.spacing_mut().item_spacing = prev_spacing;
            ui.s_offset(0., 0.);
            ui.vertical(|ui| {
                ScrollArea::vertical()
                    .id_source("ei_scroll")
                    .max_height(ui.available_height() - 30.)
                    .show(ui, |ui| {
                        ui.set_height(ui.available_height());
                        ui.set_width(ui.available_width());
                        self.item(ui);
                    });
                ui.separator();
                self.addition(ui);
            });
        });

        ui.ctx().set_data("ei_selected", self.selected);
    }

    fn list(&mut self, ui: UiRef) {
        let mut sorted: Vec<_> = self
            .items
            .iter()
            .enumerate()
            .map(|(idx, i)| (idx, i, i.get_name()))
            .collect();
        sorted.sort_by_key(|i| i.2);
        let get_nth_idx = |idx| sorted.get(idx).map(|(idx, _, _)| *idx).unwrap_or_default();
        self.selected = ui
            .ctx()
            .get_data("ei_selected")
            .unwrap_or_else(|| get_nth_idx(0));

        let scroll_to: Option<usize> = ui.ctx().get_data("ei_scroll_to");

        let mut removed = None;
        // can't go lower than the first, so go for the second
        let mut prev_idx = get_nth_idx(1);
        for (idx, item, name) in sorted {
            ui.horizontal(|ui| {
                if ui.s_icon_button(Icon::Remove, "Remove").clicked() {
                    removed = Some((idx, prev_idx));
                }
                prev_idx = idx;
                ui.s_offset(5., 0.);
                let r = ui.s_list_item(idx == self.selected, color_text(name, WHITE));
                on_hover_text_side(ui, &r, &item.tag);
                if r.clicked() {
                    self.selected = idx;
                }

                if scroll_to == Some(idx) {
                    ui.scroll_to_rect(r.rect, Some(Align::Center));
                    ui.ctx().remove_data::<usize>("ei_scroll_to");
                }
            });
        }
        if let Some((idx, prev_idx)) = removed {
            self.items.remove(idx);
            // if deleted currently selected, go to one before it
            if self.selected == idx {
                self.selected = prev_idx;
            }
            // selected was after the deleted and the index has shifted by 1
            if self.selected > idx {
                self.selected -= 1;
            }
        }
    }

    fn item(&mut self, ui: UiRef) {
        if self.items.is_empty() {
            return;
        }
        let item = &mut self.items[self.selected];
        let data_item = self.data.items.get(&item.tag);
        set_striped_styles(ui);
        Grid::new(ui.next_auto_id())
            .striped(true)
            .min_col_width(80.)
            .num_columns(2)
            .spacing([10., 6.])
            .show(ui, |ui| {
                if let Some(name) = &item.name {
                    ui.label("Name:");
                    ui.s_text(name);
                    ui.end_row();
                } else if let Some(name) = data_item.and_then(|i| i.name.as_ref()) {
                    ui.add(Label::new("Template name:").wrap(true));
                    ui.s_text(name);
                    ui.end_row();
                }

                ui.label("Tag:");
                ui.s_text(&item.tag);
                ui.end_row();

                ui.label("Stack size:");
                set_slider_styles(ui);
                ui.s_slider(&mut item.stack_size, 0..=99, false);
                ui.end_row();

                if item.max_charges > 0 {
                    ui.label(color_text("Charges:", GREEN));
                    ui.s_slider(&mut item.charges, 1..=item.max_charges, false);
                    ui.end_row();
                }

                if let Some(description) = &item.description {
                    ui.label(color_text("Description:", GREEN));
                    ui.add(Label::new(color_text(description, WHITE)).wrap(true));
                    ui.end_row();
                } else if let Some(description) = data_item.and_then(|i| i.description.as_ref()) {
                    ui.add(Label::new(color_text("Template description:", GREEN)).wrap(true));
                    ui.add(Label::new(color_text(description, WHITE)).wrap(true));
                    ui.end_row();
                }
            });
    }

    fn addition(&mut self, ui: UiRef) {
        let show_all = ui.ctx().get_data("ei_add_all").unwrap_or(false);
        ui.horizontal(|ui| {
            ui.label("Add item:");
            set_combobox_styles(ui);
            ComboBox::from_id_source("ei_add")
                .width(300.)
                .show_ui(ui, |ui| {
                    set_selectable_styles(ui);
                    let mut selected = None;
                    for item in &self.data.inner.items {
                        if !show_all && item.name.is_none() {
                            continue;
                        }
                        let r = ui.selectable_value(&mut selected, Some(&item.id), item.get_name());
                        on_hover_text_side(ui, &r, &item.tag);
                    }
                    if let Some(id) = selected {
                        let item = &self.data.items[id];
                        let idx = self.items.len();
                        self.items.push(Item {
                            tag: item.tag.clone(),
                            name: item.name.clone(),
                            description: item.description.clone(),
                            stack_size: item.stack_size,
                            max_charges: item.charges,
                            charges: item.charges,
                            new: true,
                            upgrades: 0,
                            upgrade_slots: item.upgrade_level.map(|_| [-1; 6]),
                            raw: item.inner.clone(),
                        });
                        self.selected = idx;
                        ui.ctx().set_data("ei_scroll_to", idx);
                    }
                });
            let mut checked = show_all;
            ui.label(color_text("Show all: ", GREEN));
            set_checkbox_styles(ui);
            ui.s_checkbox(&mut checked);
            if checked != show_all {
                ui.ctx().set_data("ei_add_all", checked);
            }
        });
    }
}
