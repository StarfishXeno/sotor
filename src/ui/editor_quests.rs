use crate::save::JournalEntry;
use egui::{DragValue, Grid, RichText, ScrollArea, TextStyle};
use std::sync::{Arc, Mutex};

use super::{
    styles::{
        set_button_styles, set_button_styles_disabled, set_drag_value_styles, set_striped_styles,
    },
    widgets::{white_text, UiExt},
    UiRef,
};

fn area(ui: UiRef, idx: u8, add_contents: impl FnOnce(UiRef)) {
    ScrollArea::vertical()
        .id_source(format!("editor_quests_scroll_{idx}"))
        .show(ui, |ui| {
            set_striped_styles(ui);

            Grid::new(format!("editor_quests_grid_{idx}"))
                .num_columns(7)
                .spacing([5.0, 5.0])
                .striped(true)
                .show(ui, add_contents);
        });
}

fn row(ui: UiRef, entry: &mut JournalEntry, removed: &mut Option<usize>, idx: usize) {
    ui.label(white_text(&entry.id).text_style(TextStyle::Small));
    set_drag_value_styles(ui);
    ui.add(DragValue::new(&mut entry.state));
    set_button_styles(ui);
    let btn = ui.s_button_basic("Remove");
    if btn.clicked() {
        *removed = Some(idx);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EditorQuestsState {
    id: String,
    state: i32,
}

pub struct EditorQuests<'a> {
    journal: &'a mut Vec<JournalEntry>,
}
impl<'a> EditorQuests<'a> {
    pub fn new(journal: &'a mut Vec<JournalEntry>) -> Self {
        Self { journal }
    }

    pub fn show(&mut self, ui: UiRef) {
        area(ui, 0, |ui| {
            self.table(ui);
        });

        ui.separator();
        ui.horizontal(|ui| self.addition(ui));
    }

    fn table(&mut self, ui: UiRef) {
        ui.label(RichText::new("Name").underline());
        ui.label(RichText::new("State").underline());
        ui.label("");
        ui.label("");
        ui.label(RichText::new("Name").underline());
        ui.label(RichText::new("State").underline());
        ui.label("");
        ui.end_row();
        set_drag_value_styles(ui);

        let mut removed = None;
        for (chunk_idx, entries) in self.journal.chunks_mut(2).enumerate() {
            let idx = chunk_idx * 2;
            row(ui, &mut entries[0], &mut removed, idx);
            if entries.len() > 1 {
                ui.label("");
                row(ui, &mut entries[1], &mut removed, idx + 1);
            }
            ui.end_row();
        }
        if let Some(idx) = removed {
            self.journal.remove(idx);
        }
    }

    fn addition(&mut self, ui: UiRef) {
        let state = ui.ctx().memory_mut(|mem| {
            let data = &mut mem.data;

            let state = data
                .get_temp_mut_or_insert_with("editor_quests".into(), || {
                    Arc::new(Mutex::new(EditorQuestsState {
                        id: String::new(),
                        state: 0,
                    }))
                })
                .clone();

            state
        });
        let mut state = state.lock().unwrap();

        ui.label("Quest name: ");
        ui.s_text_edit(&mut state.id, 150.0);
        ui.label("State: ");
        set_drag_value_styles(ui);
        ui.add(DragValue::new(&mut state.state));

        let disabled = state.id.is_empty();
        set_button_styles(ui);
        if disabled {
            set_button_styles_disabled(ui);
        }
        let btn = ui.s_button("Add", false, disabled);
        if !disabled && btn.clicked() {
            let last = self.journal.last();
            self.journal.push(JournalEntry {
                id: state.id.clone(),
                state: state.state,
                date: last.map_or(0, |e| e.date),
                time: last.map_or(0, |e| e.time + 1),
            });
            state.id = String::new();
            state.state = 0;
        }
    }
}
