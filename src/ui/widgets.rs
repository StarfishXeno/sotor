use super::styles::{set_checkbox_styles, set_slider_styles, BLACK, GREEN, WHITE};
use egui::{
    epaint::TextShape, style::HandleShape, Button, Color32, CursorIcon, FontSelection, Response,
    RichText, Sense, Slider, Stroke, TextBuffer, TextEdit, TextStyle, Ui, Widget, WidgetInfo,
    WidgetText, WidgetType,
};
use emath::{Align, Numeric, Rect};
use std::ops::RangeInclusive;

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
    fn s_slider<T: Numeric>(&mut self, value: &mut T, range: RangeInclusive<T>);
    fn s_button(&mut self, text: &str, selected: bool, disabled: bool) -> Response;
    fn s_button_basic(&mut self, text: &str) -> Response;
    fn s_checkbox_raw(&mut self, value: &mut bool) -> Response;
    fn s_checkbox(&mut self, value: &mut bool);
    fn s_text(&mut self, text: &str) -> Response;
    fn s_scroll_to_end(&mut self);
    fn s_offset(&mut self, offset: [f32; 2]);
    fn s_icon_button(&mut self, icon: Icon, hint: &str) -> Response;
    fn s_icon_button_raw(&mut self, icon: Icon, hint: Option<&str>, size: f32) -> Response;
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

    fn s_slider<T: Numeric>(&mut self, value: &mut T, range: RangeInclusive<T>) {
        self.horizontal(|ui| {
            set_slider_styles(ui);

            ui.s_slider_raw(value, range, true);
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

    fn s_scroll_to_end(&mut self) {
        self.scroll_to_rect(
            Rect::from_min_max([f32::MAX, f32::MAX].into(), [f32::MAX, f32::MAX].into()),
            None,
        );
    }

    fn s_offset(&mut self, offset: [f32; 2]) {
        self.allocate_exact_size(offset.into(), Sense::hover());
    }

    fn s_icon_button_raw(&mut self, icon: Icon, hint: Option<&str>, size: f32) -> Response {
        let btn = IconButton {
            icon,
            hint,
            color: GREEN,
            color_hovered: WHITE,
            size,
        };

        self.add(btn)
    }

    fn s_icon_button(&mut self, icon: Icon, hint: &str) -> Response {
        self.s_icon_button_raw(icon, Some(hint), 18.0)
    }
}

pub enum Icon {
    Gear,
    Reload,
    Refresh,
    Save,
    Close,
}

impl Icon {
    fn get(self) -> &'static str {
        match self {
            Self::Gear => "\u{f013}",
            Self::Reload => "\u{f079}",
            Self::Refresh => "\u{f021}",
            Self::Save => "\u{f0c7}",
            Self::Close => "\u{f00d}",
        }
    }
}

struct IconButton<'a> {
    icon: Icon,
    hint: Option<&'a str>,
    color: Color32,
    color_hovered: Color32,
    size: f32,
}

impl<'a> Widget for IconButton<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let valign = ui.layout().vertical_align();
        let text: WidgetText = RichText::new(self.icon.get())
            .text_style(TextStyle::Name("icon".into()))
            .size(self.size)
            .into();
        let mut text_job = text.into_text_job(ui.style(), FontSelection::Default, valign);

        text_job.job.wrap.max_width = f32::INFINITY;
        text_job.job.halign = ui.layout().horizontal_placement();
        text_job.job.justify = ui.layout().horizontal_justify();

        let text_galley = ui.fonts(|f| text_job.into_galley(f));
        let (rect, mut response) = ui.allocate_exact_size(text_galley.size(), Sense::click());
        let pos = match text_galley.galley.job.halign {
            Align::LEFT => rect.left_top(),
            Align::Center => rect.center_top(),
            Align::RIGHT => rect.right_top(),
        };

        response.widget_info(|| WidgetInfo::labeled(WidgetType::Other, text_galley.text()));

        if ui.is_rect_visible(response.rect) {
            let override_text_color = if response.hovered() {
                Some(self.color_hovered)
            } else {
                Some(self.color)
            };

            ui.painter().add(TextShape {
                pos,
                galley: text_galley.galley,
                override_text_color,
                underline: Stroke::NONE,
                angle: 0.0,
            });
        }

        if let Some(hint) = self.hint {
            response = response.on_hover_text(hint);
        }
        response = response.on_hover_cursor(CursorIcon::PointingHand);

        response
    }
}
