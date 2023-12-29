use crate::{
    save::JournalEntry,
    ui::{
        styles::{
            set_button_styles, set_button_styles_disabled, set_drag_value_styles,
            set_striped_styles,
        },
        widgets::{white_text, UiExt},
        UiRef,
    },
    util::ContextExt as _,
};
use egui::{DragValue, Grid, Label, RichText, ScrollArea, TextStyle};
use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

#[derive(Debug, Clone, PartialEq)]
pub struct EditorQuestsState {
    id: String,
    state: i32,
}
const QUESTS_STATE_ID: &str = "editor_quests_state";
const COLUMN_WIDTH: f32 = 300.;
pub struct EditorQuests<'a> {
    journal: &'a mut Vec<JournalEntry>,
    width: f32,
}
impl<'a> EditorQuests<'a> {
    pub fn new(journal: &'a mut Vec<JournalEntry>) -> Self {
        Self { journal, width: 0. }
    }

    pub fn show(&mut self, ui: UiRef) {
        self.width = ui.available_width();

        Self::area(ui, |ui| {
            self.table(ui);
        });
        ui.separator();
        ui.horizontal(|ui| self.addition(ui));
    }

    fn table(&mut self, ui: UiRef) {
        let columns = (self.width / COLUMN_WIDTH) as u32;
        let mut count = 0;

        for _ in 0..columns {
            ui.label(RichText::new("Name").underline());
            ui.label(RichText::new("State").underline());
            ui.label("");
            ui.s_offset([5.0, 0.]);
        }

        ui.end_row();
        set_drag_value_styles(ui);

        let mut removed = None;
        for (idx, entry) in self.journal.iter_mut().enumerate() {
            Self::column(ui, entry, &mut removed, idx);

            if count == columns - 1 {
                ui.end_row();
                count = 0;
            } else {
                count += 1;
            }
        }

        if let Some(idx) = removed {
            self.journal.remove(idx);
        }
    }

    fn addition(&mut self, ui: UiRef) {
        let state = ui.ctx().get_data(QUESTS_STATE_ID).unwrap_or_else(|| {
            let state = Arc::new(Mutex::new(EditorQuestsState {
                id: String::new(),
                state: 0,
            }));
            ui.ctx().set_data(QUESTS_STATE_ID, state.clone());

            state
        });
        let mut state = state.lock().unwrap();

        ui.label("Quest name: ");
        ui.s_text_edit(&mut state.id, 150.0);
        ui.label("State: ");
        set_drag_value_styles(ui);
        ui.add(DragValue::new(&mut state.state));

        let set: HashSet<_> = self.journal.iter().map(|e| &e.id).collect();
        let already_there = set.contains(&state.id);
        let disabled = state.id.trim().is_empty() || already_there;
        set_button_styles(ui);
        if disabled {
            set_button_styles_disabled(ui);
        }
        let mut btn = ui.s_button("Add", false, disabled);
        if already_there {
            btn = btn.on_hover_text("This quest is already present");
        }
        if !disabled && btn.clicked() {
            let last = self.journal.last();
            self.journal.push(JournalEntry {
                id: state.id.trim().to_owned(),
                state: state.state,
                date: last.map_or(0, |e| e.date),
                time: last.map_or(0, |e| e.time + 1),
            });
            state.id = String::new();
            state.state = 0;
        }
    }

    fn area(ui: UiRef, add_contents: impl FnOnce(UiRef)) {
        ScrollArea::vertical()
            .id_source("editor_quests_scroll")
            .stick_to_bottom(true)
            .max_height(ui.max_rect().height() - 65.0)
            .show(ui, |ui| {
                set_striped_styles(ui);

                Grid::new("editor_quests_grid")
                    .spacing([5.0, 5.0])
                    .striped(true)
                    .max_col_width(COLUMN_WIDTH - 110.)
                    .min_col_width(0.)
                    .show(ui, add_contents);
            });
    }

    fn column(ui: UiRef, entry: &mut JournalEntry, removed: &mut Option<usize>, idx: usize) {
        ui.add(Label::new(white_text(&entry.id).text_style(TextStyle::Small)).wrap(true));
        set_drag_value_styles(ui);
        ui.add(DragValue::new(&mut entry.state));
        set_button_styles(ui);
        let btn = ui.s_button_basic("Remove");
        if btn.clicked() {
            *removed = Some(idx);
        }
        ui.s_offset([0.0, 0.]);
    }
}
