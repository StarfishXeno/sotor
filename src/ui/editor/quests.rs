use crate::{
    save::{JournalEntry, Save},
    ui::{
        styles::{
            set_combobox_styles, set_drag_value_styles, set_selectable_styles, set_striped_styles,
            GREEN, GREEN_DARK, GREY, WHITE,
        },
        widgets::{color_text, Icon, IconButton, UiExt},
        UiRef,
    },
    util::ContextExt as _,
};
use egui::{
    epaint::TextShape, Area, ComboBox, DragValue, FontSelection, Frame, Grid, Label, Order,
    RichText, ScrollArea, Sense, WidgetText,
};
use emath::{pos2, vec2, Align, Rect};
use internal::{util::shorten_string, GameDataMapped, Quest, QuestStage};
use std::{
    collections::HashSet,
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
            .max_height(ui.available_height() - 35.)
            .show(ui, |ui| {
                set_striped_styles(ui);
                ui.set_width(self.width);

                Grid::new("eq_grid")
                    .spacing([5., 5.])
                    .striped(true)
                    .num_columns(3)
                    .max_col_width(210.)
                    .show(ui, |ui| {
                        self.table(ui);
                    });
            });
        ui.separator();
        ui.horizontal(|ui| self.addition(ui));
    }

    fn table(&mut self, ui: UiRef) {
        ui.s_empty();
        ui.label(RichText::new("Name").underline());
        ui.label(RichText::new("Stage").underline());
        ui.end_row();
        set_combobox_styles(ui);

        let mut quests = Vec::with_capacity(self.journal.len());
        for (idx, entry) in self.journal.iter_mut().enumerate() {
            let quest = self.data.quests.get(&entry.id.to_lowercase());
            let stage = quest.and_then(|q| q.stages.get(&entry.stage));
            let name = quest.map_or(entry.id.clone(), |q| q.name.clone());
            let completed = stage.map_or(false, |s| s.end);

            quests.push((completed, name, entry, quest, stage, idx));
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
        for (completed, name, entry, quest, stage, idx) in quests {
            let remove = || removed = Some(idx);
            Self::quest(ui, completed, name.as_str(), entry, quest, stage, remove);
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
        quest: Option<&Quest>,
        stage: Option<&QuestStage>,
        remove: impl FnOnce(),
    ) {
        if ui.s_icon_button(Icon::Remove, "Remove").clicked() {
            remove();
        }
        if let Some(quest) = quest {
            let name_color = if completed { GREY } else { WHITE };
            let name_text = Label::new(color_text(name, name_color).small());
            ui.add(name_text)
                .on_hover_text(color_text(&quest.id, WHITE));
            let stage_name =
                stage.map_or_else(|| entry.stage.to_string() + ") UNKNOWN", |s| s.get_name(60));

            let r = ComboBox::from_id_source(&quest.id)
                .width(400.)
                .selected_text(RichText::new(stage_name).small())
                .show_ui(ui, |ui| {
                    set_selectable_styles(ui);
                    let mut selected = entry.stage;
                    for (id, stage) in &quest.stages {
                        let name = RichText::new(stage.get_name(60)).small();
                        let r = ui.selectable_value(&mut selected, *id, name);
                        if r.hovered() {
                            Self::show_description(ui, r.rect, stage);
                        }
                    }
                    entry.stage = selected;
                });
            if let Some(stage) = stage {
                if r.response.hovered() {
                    Self::show_description(ui, r.response.rect, stage);
                }
            }
        } else {
            ui.add(Label::new(color_text(name, WHITE).small()).wrap(true));
            set_drag_value_styles(ui);
            ui.add(DragValue::new(&mut entry.stage));
        }

        ui.end_row();
    }

    fn addition(&mut self, ui: UiRef) {
        const QUESTS_STATE_ID: &str = "eq_state";
        let get_present_ids = || self.journal.iter().map(|e| &e.id).collect::<HashSet<_>>();
        let state = ui.ctx().get_data(QUESTS_STATE_ID).unwrap_or_else(|| {
            let present = get_present_ids();
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
            .width(240.)
            .selected_text(shorten_string(name, 30))
            .show_ui(ui, |ui| {
                set_selectable_styles(ui);
                let mut selected = &current_quest.map(|q| q.id.clone()).unwrap_or_default();
                let present = get_present_ids();
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
                    if r.hovered() {
                        Self::show_description(ui, r.rect, stage);
                    }
                }
                state.stage = selected;
            });
        if let Some(stage) = current_stage {
            if r.response.hovered() {
                Self::show_description(ui, r.response.rect, stage);
            }
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

    // this is a mess, but it's better than normal .on_hover_text
    // TODO simplify, somehow
    fn show_description(ui: UiRef, rect: Rect, stage: &QuestStage) {
        let description = shorten_string(&stage.description, 1000);
        if description.is_empty() {
            return;
        }
        let anchor = rect.left_bottom();
        let ctx = ui.ctx();
        let style = ui.style();
        let screen_rect = ctx.screen_rect();
        let mut layout_job = WidgetText::RichText(color_text(&description, WHITE)).into_layout_job(
            ui.style(),
            FontSelection::Default,
            Align::Min,
        );
        layout_job.wrap.max_width = anchor.x - 50.;

        let galley = ui.fonts(|f| f.layout_job(layout_job));
        let size = galley.size() + style.spacing.menu_margin.sum();
        let y_offset = 'b: {
            let below = rect.left_top().y;
            if below + size.y < screen_rect.max.y {
                break 'b below;
            }
            let above = anchor.y - size.y;
            if above > 0. {
                break 'b above;
            }
            screen_rect.max.y / 2. - size.y / 2.
        };
        let pos = screen_rect.left_top() + vec2(anchor.x - 10. - size.x, y_offset);

        Area::new("eq_descr")
            .order(Order::Tooltip)
            .fixed_pos(pos)
            .interactable(false)
            .show(ctx, |ui| {
                Frame::popup(style)
                    .stroke((2.0, GREEN_DARK))
                    .show(ui, |ui| {
                        ui.set_max_width(ui.max_rect().width());
                        let pos = pos2(ui.max_rect().left(), ui.cursor().top());
                        for row in &galley.rows {
                            let rect = row.rect.translate(vec2(pos.x, pos.y));
                            ui.allocate_rect(rect, Sense::hover());
                        }
                        ui.painter().add(TextShape::new(pos, galley, GREEN));
                    });
            });
    }
}
