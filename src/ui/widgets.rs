use std::ops::RangeInclusive;

use egui::{
    style::HandleShape, Button, CursorIcon, Response, RichText, Slider, TextBuffer, TextEdit, Ui,
    Widget as _,
};
use emath::{Numeric, Vec2};

use super::styles::{set_checkbox_styles, set_slider_styles, BLACK, WHITE};

pub trait UiExt {
    fn s_text_edit(&mut self, text: &mut dyn TextBuffer) -> Response;
    fn s_slider<T: Numeric>(&mut self, value: &mut T, range: RangeInclusive<T>);
    fn s_button(&mut self, text: &str, selected: bool, disabled: bool) -> Response;
    fn s_checkbox(&mut self, value: &mut bool, text: &str);
}

impl UiExt for Ui {
    fn s_text_edit(&mut self, text: &mut dyn TextBuffer) -> Response {
        TextEdit::singleline(text)
            .vertical_align(egui::Align::Center)
            .min_size(Vec2::new(300.0, 0.0))
            .text_color(BLACK)
            .ui(self)
    }
    fn s_slider<T: Numeric>(&mut self, value: &mut T, range: RangeInclusive<T>) {
        self.horizontal(|ui| {
            set_slider_styles(ui);
            ui.add(
                Slider::new(value, range)
                    .logarithmic(true)
                    .handle_shape(HandleShape::Rect { aspect_ratio: 1.0 })
                    .clamp_to_range(true),
            )
        });
    }

    fn s_button(&mut self, text: &str, selected: bool, disabled: bool) -> Response {
        let mut text = RichText::new(text);
        if selected {
            text = text.color(WHITE);
        }
        let mut btn = Button::new(text).selected(selected).rounding(2.0);
        if selected {
            btn = btn.stroke((2.0, WHITE));
        }
        btn.ui(self).on_hover_cursor(if disabled {
            CursorIcon::NotAllowed
        } else {
            CursorIcon::PointingHand
        })
    }
    fn s_checkbox(&mut self, value: &mut bool, text: &str) {
        self.horizontal(|ui| {
            set_checkbox_styles(ui);
            ui.checkbox(value, text)
                .on_hover_cursor(CursorIcon::PointingHand);
        });
    }
}
