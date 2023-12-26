use crate::{
    save::{Globals, PartyMember, Save},
    util::format_seconds,
};
use egui::{
    ComboBox, DragValue, Frame, Grid, Image, Margin, RichText, ScrollArea, Sense, TextStyle,
    TextureHandle,
};

use super::{
    styles::{
        set_button_styles, set_button_styles_disabled, set_checkbox_styles, set_combobox_styles,
        set_drag_value_styles, set_selectable_styles, set_slider_styles, GREEN, GREY_DARK, WHITE,
    },
    widgets::{white_text, UiExt},
    UiRef,
};

pub struct EditorGlobals<'a> {
    globals: &'a mut Globals,
}
impl<'a> EditorGlobals<'a> {
    pub fn new(globals: &'a mut Globals) -> Self {
        Self { globals }
    }
    pub fn show(&mut self, ui: UiRef) {
        ui.horizontal_top(|ui| {
            Self::area("globals_numbers", ui, |ui| {
                self.numbers(ui);
            });
            Self::area("globals_booleans", ui, |ui| {
                self.booleans(ui);
            });
            Self::area("globals_strings", ui, |ui| {
                self.strings(ui);
            });
        });
    }

    fn area(id: &str, ui: UiRef, add_contents: impl FnOnce(UiRef)) {
        ScrollArea::vertical()
            .id_source(id.to_owned() + "_scroll")
            .show(ui, |ui| {
                Grid::new(id)
                    .num_columns(2)
                    .spacing([5.0, 5.0])
                    .show(ui, add_contents);
            });
    }

    fn numbers(&mut self, ui: UiRef) {
        set_drag_value_styles(ui);

        for global in &mut self.globals.numbers {
            ui.label(white_text(&global.name).text_style(TextStyle::Small));
            ui.horizontal(|ui| {
                ui.add(DragValue::new(&mut global.value));
                // ghetto padding
                ui.label("");
                ui.label("");
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
