use crate::{
    save::Globals,
    ui::{
        styles::{set_checkbox_styles, set_drag_value_styles, set_striped_styles},
        widgets::{white_text, UiExt},
        UiRef,
    },
};
use egui::{DragValue, Grid, ScrollArea, TextStyle};

fn area(id: &str, ui: UiRef, add_contents: impl FnOnce(UiRef)) {
    ScrollArea::vertical()
        .id_source(id.to_owned() + "_scroll")
        .show(ui, |ui| {
            set_striped_styles(ui);

            Grid::new(id)
                .num_columns(2)
                .spacing([5.0, 5.0])
                .striped(true)
                .show(ui, add_contents);
        });
}

pub struct EditorGlobals<'a> {
    globals: &'a mut Globals,
}
impl<'a> EditorGlobals<'a> {
    pub fn new(globals: &'a mut Globals) -> Self {
        Self { globals }
    }
    pub fn show(&mut self, ui: UiRef) {
        ui.horizontal_top(|ui| {
            area("globals_numbers", ui, |ui| self.numbers(ui));
            area("globals_booleans", ui, |ui| self.booleans(ui));
            area("globals_strings", ui, |ui| self.strings(ui));
        });
    }

    fn numbers(&mut self, ui: UiRef) {
        set_drag_value_styles(ui);

        for global in &mut self.globals.numbers {
            ui.label(white_text(&global.name).text_style(TextStyle::Small));
            ui.horizontal(|ui| {
                ui.add(DragValue::new(&mut global.value));
                // ghetto padding
                ui.label(" ");
            });
            ui.end_row();
        }
    }

    fn booleans(&mut self, ui: UiRef) {
        set_checkbox_styles(ui);
        for global in &mut self.globals.booleans {
            ui.label(white_text(&global.name).text_style(TextStyle::Small));
            ui.s_checkbox_raw(&mut global.value);
            ui.end_row();
        }
    }

    fn strings(&mut self, ui: UiRef) {
        for global in &mut self.globals.strings {
            ui.label(white_text(&global.name).text_style(TextStyle::Small));
            ui.s_text_edit(&mut global.value, 100.0);
            ui.end_row();
        }
    }
}
