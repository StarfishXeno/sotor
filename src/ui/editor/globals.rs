use crate::{
    save::{Global, GlobalValue},
    ui::{
        styles::{set_checkbox_styles, set_drag_value_styles, set_striped_styles},
        widgets::{white_text, UiExt},
        UiRef,
    },
    util::ColumnCounter,
};
use egui::{DragValue, Grid, ScrollArea, TextStyle};

pub struct EditorGlobals<'a> {
    globals: &'a mut Vec<Global>,
    width: f32,
}

impl<'a> EditorGlobals<'a> {
    pub fn new(globals: &'a mut Vec<Global>) -> Self {
        Self { globals, width: 0. }
    }

    pub fn show(&mut self, ui: UiRef) {
        self.width = ui.available_width();

        ScrollArea::vertical()
            .id_source("editor_globals_scroll")
            .show(ui, |ui| {
                set_striped_styles(ui);

                Grid::new("editor_globals_grid")
                    .spacing([5.0, 5.0])
                    .striped(true)
                    .show(ui, |ui| self.globals(ui));
            });
    }

    fn globals(&mut self, ui: UiRef) {
        const COLUMN_WIDTH: f32 = 220.;
        let mut counter = ColumnCounter::new((self.width / COLUMN_WIDTH) as u32);

        for global in &mut *self.globals {
            match &mut global.value {
                GlobalValue::Number(value) => Self::number(ui, &global.name, value),
                GlobalValue::Boolean(value) => Self::boolean(ui, &global.name, value),
            }
            counter.next(ui);
        }
    }

    fn number(ui: UiRef, name: &str, value: &mut u8) {
        set_drag_value_styles(ui);

        ui.label(white_text(name).text_style(TextStyle::Small));
        ui.horizontal(|ui| {
            ui.add(DragValue::new(value));
            // ghetto padding
            ui.label(" ");
        });
    }

    fn boolean(ui: UiRef, name: &str, value: &mut bool) {
        set_checkbox_styles(ui);
        ui.label(white_text(name).text_style(TextStyle::Small));
        ui.s_checkbox_raw(value);
    }
}
