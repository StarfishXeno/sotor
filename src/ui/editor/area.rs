use crate::{
    save::{Door, Save},
    ui::{
        styles::{set_checkbox_styles, set_slider_styles, set_striped_styles, GREEN_DARK},
        widgets::UiExt,
        UiRef,
    },
    util::ColumnCounter,
};
use egui::{Frame, Grid, Margin, RichText, ScrollArea};

pub struct Editor<'a> {
    doors: &'a mut Option<Vec<Door>>,
    width: f32,
}

impl<'a> Editor<'a> {
    pub fn new(save: &'a mut Save) -> Self {
        Self {
            doors: &mut save.doors,
            width: 0.,
        }
    }

    pub fn show(&mut self, ui: UiRef) {
        if self.doors.is_none() {
            ui.horizontal_centered(|ui| {
                ui.s_offset(ui.max_rect().width() / 2. - 150., 0.);
                ui.label("Area editing isn't available for this save");
            });
            return;
        };

        self.width = ui.available_width();

        ScrollArea::vertical()
            .id_source("ea_scroll")
            .show(ui, |ui| {
                set_striped_styles(ui);
                ui.set_width(self.width);

                ui.label("Doors: ");

                Frame::default()
                    .rounding(2.)
                    .stroke((2., GREEN_DARK))
                    .inner_margin(Margin::same(6.))
                    .show(ui, |ui| {
                        Grid::new("ea_grid")
                            .spacing([15., 5.])
                            .max_col_width(150.)
                            .striped(true)
                            .show(ui, |ui| self.doors(ui));
                    });
            });
    }

    fn doors(&mut self, ui: UiRef) {
        let doors = self.doors.as_mut().unwrap();
        let columns = (((self.width - 12.) / 330.) as usize).clamp(1, doors.len().max(1));
        let mut counter = ColumnCounter::new(columns);
        for _ in 0..columns {
            ui.label(RichText::new("Tag").underline());
            ui.label(RichText::new("Locked").underline());
            ui.label(RichText::new("Open state").underline());
            counter.next(ui);
        }

        for door in doors {
            ui.s_text(&door.tag);
            set_checkbox_styles(ui);
            ui.s_checkbox(&mut door.locked);
            set_slider_styles(ui);
            ui.s_slider(&mut door.open_state, 0..=2, false);
            counter.next(ui);
        }
    }
}
