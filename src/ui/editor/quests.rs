use crate::{
    save::{JournalEntry, Save},
    ui::{
        styles::{set_drag_value_styles, set_striped_styles, RED, WHITE},
        widgets::{color_text, Icon, IconButton, UiExt},
        UiRef,
    },
    util::{ColumnCounter, ContextExt as _},
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
pub struct Editor<'a> {
    journal: &'a mut Vec<JournalEntry>,
    width: f32,
}
impl<'a> Editor<'a> {
    pub fn new(save: &'a mut Save) -> Self {
        Self {
            journal: &mut save.party_table.journal,
            width: 0.,
        }
    }

    pub fn show(&mut self, ui: UiRef) {
        self.width = ui.available_width();

        ScrollArea::vertical()
            .id_source("editor_quests_scroll")
            .stick_to_bottom(true)
            .max_height(ui.max_rect().height() - 85.)
            .show(ui, |ui| {
                set_striped_styles(ui);

                Grid::new("editor_quests_grid")
                    .spacing([5., 5.])
                    .striped(true)
                    .max_col_width(COLUMN_WIDTH - 95.)
                    .min_col_width(0.)
                    .show(ui, |ui| {
                        self.table(ui);
                    });
            });
        ui.separator();
        ui.horizontal(|ui| self.addition(ui));
    }

    fn table(&mut self, ui: UiRef) {
        let columns = (self.width / COLUMN_WIDTH) as u32;

        for _ in 0..columns {
            ui.s_empty();
            ui.label(RichText::new("Name").underline());
            ui.label(RichText::new("State").underline());
            ui.s_empty();
        }

        ui.end_row();
        set_drag_value_styles(ui);

        let mut removed = None;
        let mut counter = ColumnCounter::new(columns);
        for (idx, entry) in self.journal.iter_mut().enumerate() {
            Self::column(ui, entry, &mut removed, idx);
            counter.next(ui);
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
        ui.s_text_edit(&mut state.id, 150.);
        ui.label("State: ");
        set_drag_value_styles(ui);
        ui.add(DragValue::new(&mut state.state));

        let set: HashSet<_> = self.journal.iter().map(|e| &e.id).collect();
        let already_there = set.contains(&state.id);
        let enabled = !state.id.trim().is_empty() && !already_there;
        let btn = ui.add_enabled(enabled, IconButton::new(Icon::Plus));

        if btn.clicked() {
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

        if already_there {
            ui.label(color_text("This quest is already present", RED));
        }
    }

    fn column(ui: UiRef, entry: &mut JournalEntry, removed: &mut Option<usize>, idx: usize) {
        let btn = ui.s_icon_button(Icon::Remove, "Remove");
        if btn.clicked() {
            *removed = Some(idx);
        }
        ui.add(Label::new(color_text(&entry.id, WHITE).text_style(TextStyle::Small)).wrap(true));
        set_drag_value_styles(ui);
        ui.add(DragValue::new(&mut entry.state));
        ui.s_empty();
    }
}
