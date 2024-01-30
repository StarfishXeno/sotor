use crate::{
    save::{JournalEntry, Save},
    ui::{
        styles::{
            set_combobox_styles, set_drag_value_styles, set_selectable_styles, set_striped_styles,
            GREEN, GREY, WHITE,
        },
        widgets::{color_text, on_hover_text_side, Icon, IconButton, UiExt},
        UiRef,
    },
    util::{get_data_name, ColumnCounter, ContextExt as _},
};
use core::{util::shorten_string, GameDataMapped, Quest, QuestStage};
use egui::{ComboBox, DragValue, Grid, RichText, ScrollArea};
use std::{
    collections::{BTreeMap, HashSet},
    sync::{Arc, Mutex},
};

#[derive(Debug, Clone, PartialEq)]
pub struct EditorQuestsState {
    id: String,
    stage: i32,
}
pub struct Editor<'a> {
    journal: &'a mut Vec<JournalEntry>,
    width: f32,
    data: &'a GameDataMapped,
}

impl<'a> Editor<'a> {
    pub fn new(save: &'a mut Save, data: &'a GameDataMapped) -> Self {
        Self {
            journal: &mut save.party_table.journal,
            data,
            width: 0.,
        }
    }

    pub fn show(&mut self, ui: UiRef) {
        self.width = ui.available_width();

        ScrollArea::vertical()
            .id_source("eq_scroll")
            .stick_to_bottom(true)
            .max_height(ui.available_height() - 30.)
            .show(ui, |ui| {
                ui.set_width(self.width);
                ui.set_height(ui.available_height());
                set_striped_styles(ui);

                Grid::new("eq_grid")
                    .spacing([5., 5.])
                    .striped(true)
                    .num_columns(3)
                    .max_col_width(230.)
                    .show(ui, |ui| {
                        self.table(ui);
                    });
            });
        ui.separator();
        ui.horizontal(|ui| self.addition(ui));
    }

    fn table(&mut self, ui: UiRef) {
        let columns = (self.width / 715.).max(1.) as u32;
        let counter = &mut ColumnCounter::new(columns);

        for _ in 0..columns {
            ui.s_empty();
            ui.label(RichText::new("Name").underline());
            ui.label(RichText::new("Stage").underline());
            counter.next(ui);
        }
        set_combobox_styles(ui);

        let mut quests = Vec::with_capacity(self.journal.len());
        for (idx, entry) in self.journal.iter_mut().enumerate() {
            let quest = self.data.quests.get(&entry.id.to_lowercase());
            let stages = quest.map(|q| &q.stages);
            let stage = stages.and_then(|s| s.get(&entry.stage));
            let name = get_data_name(&self.data.quests, &entry.id);
            let completed = stage.map_or(false, |s| s.end);

            quests.push((completed, name, entry, stages, idx));
        }
        // sort them by completeness -> name
        quests.sort_unstable_by(|a, b| {
            let completed_eq = a.0.cmp(&b.0);
            if completed_eq.is_ne() {
                return completed_eq;
            }
            a.1.cmp(&b.1)
        });

        let mut removed = None;
        for (completed, name, entry, stages, idx) in quests {
            let remove = || removed = Some(idx);
            Self::quest(ui, completed, &name, entry, stages, remove);
            counter.next(ui);
        }

        if let Some(idx) = removed {
            self.journal.remove(idx);
        }
    }

    fn quest(
        ui: UiRef,
        completed: bool,
        name: &str,
        entry: &mut JournalEntry,
        stages: Option<&BTreeMap<i32, QuestStage>>,
        remove: impl FnOnce(),
    ) {
        if ui.s_icon_button(Icon::Remove, "Remove").clicked() {
            remove();
        }

        let name_color = if completed { GREY } else { WHITE };
        let label_r = ui.label(color_text(name, name_color));
        // it's not already in the name
        if stages.is_some() {
            label_r.on_hover_text(color_text(&entry.id, WHITE));
        }

        if let Some(stages) = stages {
            let stage = stages.get(&entry.stage);
            let stage_name =
                stage.map_or_else(|| format!("{}) UNKNOWN", entry.stage), |s| s.get_name(60));

            let r = ComboBox::from_id_source(&entry.id)
                .width(435.)
                .selected_text(stage_name)
                .show_ui(ui, |ui| {
                    set_selectable_styles(ui);
                    let mut selected = entry.stage;
                    for (id, stage) in stages {
                        let r = ui.selectable_value(&mut selected, *id, stage.get_name(60));
                        on_hover_text_side(ui, &r, &stage.description);
                    }
                    entry.stage = selected;
                });

            if let Some(stage) = stage {
                on_hover_text_side(ui, &r.response, &stage.description);
            }
        } else {
            set_drag_value_styles(ui);
            ui.add(DragValue::new(&mut entry.stage));
        }
    }

    fn get_present_ids(&self) -> HashSet<&String> {
        self.journal.iter().map(|e| &e.id).collect()
    }

    fn addition(&mut self, ui: UiRef) {
        const QUESTS_STATE_ID: &str = "eq_state";
        let state = ui.ctx().get_data(QUESTS_STATE_ID).unwrap_or_else(|| {
            let present = self.get_present_ids();
            let mut quest = None;

            for q in &self.data.inner.quests {
                if !present.contains(&q.id) {
                    quest = Some(q);
                    break;
                }
            }
            let state = Arc::new(Mutex::new(EditorQuestsState {
                id: quest.map(|q| q.id.clone()).unwrap_or_default(),
                stage: quest.map(Quest::get_first_stage_id).unwrap_or_default(),
            }));
            ui.ctx().set_data(QUESTS_STATE_ID, state.clone());

            state
        });
        let mut state = state.lock().unwrap();
        let current_quest = &self.data.quests.get(&state.id);
        let name = current_quest.map(|q| q.name.as_str()).unwrap_or_default();

        ui.label("Quest: ");
        set_combobox_styles(ui);
        ComboBox::from_id_source("eq_new_id")
            .width(280.)
            .selected_text(shorten_string(name, 37))
            .show_ui(ui, |ui| {
                set_selectable_styles(ui);
                let mut selected = &current_quest.map(|q| q.id.clone()).unwrap_or_default();
                let present = self.get_present_ids();
                for quest in &self.data.inner.quests {
                    if present.contains(&quest.id) {
                        continue;
                    }

                    ui.selectable_value(&mut selected, &quest.id, &quest.name);
                }
                if current_quest.is_none() || current_quest.unwrap().id != *selected {
                    state.id = selected.clone();
                    state.stage = self.data.quests[selected].get_first_stage_id();
                }
            });
        ui.label(color_text("Stage: ", GREEN));

        let current_stage = current_quest.and_then(|q| q.stages.get(&state.stage));
        let r = ComboBox::from_id_source("eq_new_stage")
            .selected_text(current_stage.map(|s| s.get_name(40)).unwrap_or_default())
            .width(300.)
            .show_ui(ui, |ui| {
                let Some(stages) = current_quest.map(|q| &q.stages) else {
                    return;
                };
                set_selectable_styles(ui);
                let mut selected = state.stage;
                for (id, stage) in stages {
                    let r = ui.selectable_value(&mut selected, *id, stage.get_name(40));
                    on_hover_text_side(ui, &r, &stage.description);
                }
                state.stage = selected;
            });
        if let Some(stage) = current_stage {
            on_hover_text_side(ui, &r.response, &stage.description);
        }
        let btn = ui.add_enabled(!state.id.trim().is_empty(), IconButton::new(Icon::Plus));

        if btn.clicked() {
            let last = self.journal.last();
            self.journal.push(JournalEntry {
                id: state.id.trim().to_owned(),
                stage: state.stage,
                date: last.map_or(0, |e| e.date),
                time: last.map_or(0, |e| e.time + 1),
            });
            state.id = String::new();
            state.stage = 0;
        }
    }
}
