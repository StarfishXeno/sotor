use crate::{
    save::{Global, GlobalValue, Save},
    ui::{
        styles::{set_checkbox_styles, set_drag_value_styles, set_striped_styles},
        widgets::UiExt,
        UiRef,
    },
    util::ColumnCounter,
};
use egui::{DragValue, Grid, ScrollArea};

pub struct Editor<'a> {
    globals: &'a mut Vec<Global>,
    width: f32,
}

impl<'a> Editor<'a> {
    pub fn new(save: &'a mut Save) -> Self {
        Self {
            globals: &mut save.globals,
            width: 0.,
        }
    }

    pub fn show(&mut self, ui: UiRef) {
        self.width = ui.available_width();

        ScrollArea::vertical()
            .id_source("eg_scroll")
            .show(ui, |ui| {
                set_striped_styles(ui);
                ui.set_width(self.width);

                Grid::new("eg_grid")
                    .spacing([5., 5.])
                    .striped(true)
                    .show(ui, |ui| self.globals(ui));
            });
    }

    fn globals(&mut self, ui: UiRef) {
        const COLUMN_WIDTH: f32 = 220.;
        let mut counter = ColumnCounter::new((self.width / COLUMN_WIDTH) as usize);

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

        ui.s_text(name);
        ui.add(DragValue::new(value));
    }

    fn boolean(ui: UiRef, name: &str, value: &mut bool) {
        set_checkbox_styles(ui);
        ui.s_text(name);
        ui.s_checkbox(value);
    }
}
