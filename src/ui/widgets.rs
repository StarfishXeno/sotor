use std::ops::RangeInclusive;

use egui::{
    style::HandleShape, Button, CursorIcon, Response, RichText, Slider, TextBuffer, TextEdit, Ui,
    Widget as _,
};
use emath::Numeric;

use super::styles::{set_checkbox_styles, set_slider_styles, BLACK, WHITE};

pub fn white_text(text: &str) -> RichText {
    RichText::new(text).color(WHITE)
}

pub trait UiExt {
    fn s_text_edit(&mut self, text: &mut dyn TextBuffer, width: f32) -> Response;
    fn s_slider_raw<T: Numeric>(
        &mut self,
        value: &mut T,
        range: RangeInclusive<T>,
        logarithmic: bool,
    ) -> Response;
    fn s_slider<T: Numeric>(&mut self, value: &mut T, range: RangeInclusive<T>, logarithmic: bool);
    fn s_button(&mut self, text: &str, selected: bool, disabled: bool) -> Response;
    fn s_button_basic(&mut self, text: &str) -> Response;
    fn s_checkbox_raw(&mut self, value: &mut bool) -> Response;
    fn s_checkbox(&mut self, value: &mut bool);
    fn s_text(&mut self, text: &str) -> Response;
}

impl UiExt for Ui {
    fn s_text_edit(&mut self, text: &mut dyn TextBuffer, width: f32) -> Response {
        TextEdit::singleline(text)
            .vertical_align(egui::Align::Center)
            .min_size([width, 0.0].into())
            .margin([4.0, 1.6].into())
            .desired_width(width)
            .text_color(BLACK)
            .ui(self)
    }

    fn s_slider_raw<T: Numeric>(
        &mut self,
        value: &mut T,
        range: RangeInclusive<T>,
        logarithmic: bool,
    ) -> Response {
        self.add(
            Slider::new(value, range)
                .logarithmic(logarithmic)
                .handle_shape(HandleShape::Rect { aspect_ratio: 1.0 })
                .clamp_to_range(true),
        )
    }

    fn s_slider<T: Numeric>(&mut self, value: &mut T, range: RangeInclusive<T>, logarithmic: bool) {
        self.horizontal(|ui| {
            set_slider_styles(ui);
            ui.s_slider_raw(value, range, logarithmic);
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

    fn s_button_basic(&mut self, text: &str) -> Response {
        self.s_button(text, false, false)
    }

    fn s_checkbox_raw(&mut self, value: &mut bool) -> Response {
        self.checkbox(value, "")
            .on_hover_cursor(CursorIcon::PointingHand)
    }

    fn s_checkbox(&mut self, value: &mut bool) {
        self.horizontal(|ui| {
            set_checkbox_styles(ui);
            ui.s_checkbox_raw(value);
        });
    }

    fn s_text(&mut self, text: &str) -> Response {
        self.label(white_text(text))
    }
}
