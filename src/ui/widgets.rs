use super::{
    styles::{BLACK, BLUE, GREEN, GREEN_DARK, GREY, GREY_DARK, WHITE},
    UiRef,
};
use egui::{
    epaint::TextShape, style::HandleShape, Area, Button, Color32, CursorIcon, FontSelection, Frame,
    Order, Response, RichText, Rounding, Sense, Slider, Stroke, TextBuffer, TextEdit, TextStyle,
    Ui, Widget, WidgetInfo, WidgetText, WidgetType,
};
use emath::{pos2, vec2, Align, Numeric, Rect};
use internal::util::shorten_string;
use std::ops::RangeInclusive;

pub fn color_text(text: &str, color: Color32) -> RichText {
    RichText::new(text).color(color)
}

pub trait UiExt {
    fn s_text_edit(&mut self, text: &mut dyn TextBuffer, width: f32) -> Response;
    fn s_slider<T: Numeric>(
        &mut self,
        value: &mut T,
        range: RangeInclusive<T>,
        logarithmic: bool,
    ) -> Response;
    fn s_button(&mut self, text: &str, selected: bool, disabled: bool) -> Response;
    fn s_button_basic(&mut self, text: &str) -> Response;
    fn s_checkbox(&mut self, value: &mut bool) -> Response;
    fn s_text(&mut self, text: &str) -> Response;
    fn s_scroll_to_end(&mut self);
    fn s_offset(&mut self, x: f32, y: f32);
    fn s_empty(&mut self);
    fn s_icon_button(&mut self, icon: Icon, hint: &str) -> Response;
    fn s_list_item(&mut self, selected: bool, text: impl Into<WidgetText>) -> Response;
}

impl UiExt for Ui {
    fn s_text_edit(&mut self, text: &mut dyn TextBuffer, width: f32) -> Response {
        TextEdit::singleline(text)
            .vertical_align(egui::Align::Center)
            .min_size([width, 0.].into())
            .margin([4., 1.].into())
            .desired_width(width)
            .text_color(BLACK)
            .ui(self)
    }

    fn s_slider<T: Numeric>(
        &mut self,
        value: &mut T,
        range: RangeInclusive<T>,
        logarithmic: bool,
    ) -> Response {
        self.add(
            Slider::new(value, range)
                .logarithmic(logarithmic)
                .handle_shape(HandleShape::Rect { aspect_ratio: 1. })
                .clamp_to_range(true),
        )
    }

    fn s_button(&mut self, text: &str, selected: bool, disabled: bool) -> Response {
        let mut text = RichText::new(text);
        if selected {
            text = text.color(WHITE);
        }
        let mut btn = Button::new(text).selected(selected).rounding(2.);
        if selected {
            btn = btn.stroke((2., WHITE));
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

    fn s_checkbox(&mut self, value: &mut bool) -> Response {
        self.checkbox(value, "")
            .on_hover_cursor(CursorIcon::PointingHand)
    }

    fn s_text(&mut self, text: &str) -> Response {
        self.label(color_text(text, WHITE))
    }

    fn s_scroll_to_end(&mut self) {
        self.scroll_to_rect(
            Rect::from_min_max([f32::MAX, f32::MAX].into(), [f32::MAX, f32::MAX].into()),
            None,
        );
    }

    fn s_offset(&mut self, x: f32, y: f32) {
        self.allocate_exact_size([x, y].into(), Sense::hover());
    }

    fn s_empty(&mut self) {
        self.s_offset(0., 0.);
    }

    fn s_icon_button(&mut self, icon: Icon, hint: &str) -> Response {
        self.add(IconButton::new(icon).hint(hint))
    }

    fn s_list_item(&mut self, selected: bool, text: impl Into<WidgetText>) -> Response {
        ListItem::new(selected, text).ui(self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Icon {
    Close,
    #[cfg(not(target_arch = "wasm32"))]
    Cloud,
    #[cfg(not(target_arch = "wasm32"))]
    Gear,
    Leave,
    Plus,
    Refresh,
    #[cfg(not(target_arch = "wasm32"))]
    Reload,
    Remove,
    Save,
    #[allow(dead_code)]
    Triangle,
}

impl Icon {
    pub fn symbol(self) -> &'static str {
        match self {
            Self::Close => "\u{f00d}",
            #[cfg(not(target_arch = "wasm32"))]
            Self::Cloud => "\u{f0c2}",
            #[cfg(not(target_arch = "wasm32"))]
            Self::Gear => "\u{f013}",
            Self::Leave => "\u{f2f5}",
            Self::Plus => "\u{002b}",
            Self::Refresh => "\u{f021}",
            #[cfg(not(target_arch = "wasm32"))]
            Self::Reload => "\u{f079}",
            Self::Remove => "\u{f2ed}",
            Self::Save => "\u{f0c7}",
            Self::Triangle => "\u{f0d7}",
        }
    }
}

pub struct IconButton<'a> {
    icon: Icon,
    hint: Option<&'a str>,
    color: Color32,
    color_hovered: Color32,
    size: f32,
}

impl<'a> Default for IconButton<'a> {
    fn default() -> Self {
        Self {
            icon: Icon::Close,
            hint: None,
            color: GREEN,
            color_hovered: WHITE,
            size: 18.,
        }
    }
}

impl<'a> IconButton<'a> {
    pub fn new(icon: Icon) -> Self {
        Self {
            icon,
            ..Default::default()
        }
    }

    pub fn hint(mut self, hint: &'a str) -> Self {
        self.hint = Some(hint);
        self
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }
}

impl<'a> Widget for IconButton<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let valign = ui.layout().vertical_align();
        let text: WidgetText = RichText::new(self.icon.symbol())
            .text_style(TextStyle::Name("icon".into()))
            .size(self.size)
            .into();
        let mut layout_job = text.into_layout_job(ui.style(), FontSelection::Default, valign);

        layout_job.wrap.max_width = f32::INFINITY;
        layout_job.halign = ui.layout().horizontal_placement();
        layout_job.justify = ui.layout().horizontal_justify();

        let text_galley = ui.fonts(|f| f.layout_job(layout_job));

        let (rect, mut response) = ui.allocate_exact_size(text_galley.size(), Sense::click());
        let pos = match text_galley.job.halign {
            Align::LEFT => rect.left_top(),
            Align::Center => rect.center_top(),
            Align::RIGHT => rect.right_top(),
        };

        response.widget_info(|| WidgetInfo::labeled(WidgetType::Other, text_galley.text()));

        if ui.is_rect_visible(response.rect) {
            let override_text_color = if !ui.is_enabled() {
                Some(GREY)
            } else if response.hovered() {
                Some(self.color_hovered)
            } else {
                Some(self.color)
            };

            ui.painter().add(TextShape {
                fallback_color: GREEN,
                pos,
                galley: text_galley,
                override_text_color,
                underline: Stroke::NONE,
                angle: 0.,
            });
        }

        if let Some(hint) = self.hint {
            response = response.on_hover_text(hint);
        }
        response = response.on_hover_cursor(CursorIcon::PointingHand);

        response
    }
}

pub struct ListItem {
    selected: bool,
    text: WidgetText,
}

impl ListItem {
    pub fn new(selected: bool, text: impl Into<WidgetText>) -> Self {
        Self {
            selected,
            text: text.into(),
        }
    }
}

impl Widget for ListItem {
    fn ui(self, ui: &mut Ui) -> Response {
        let Self { selected, text } = self;

        let width = ui.available_width();
        let padding = vec2(8., 2.);

        let text_galley =
            text.into_galley(ui, Some(true), width - padding.x * 2., TextStyle::Button);

        let desired_size = [width, text_galley.size().y + padding.y].into();
        let (rect, mut response) = ui.allocate_at_least(desired_size, Sense::click());

        response
            .widget_info(|| WidgetInfo::selected(WidgetType::Other, selected, text_galley.text()));
        response = response.on_hover_cursor(CursorIcon::PointingHand);

        if ui.is_rect_visible(response.rect) {
            let pos = rect.shrink2(padding).left_top();

            if selected || response.hovered() || response.highlighted() || response.has_focus() {
                ui.painter()
                    .rect(rect, Rounding::ZERO, GREY_DARK, Stroke::NONE);
            }

            let override_text_color = if selected { Some(BLUE) } else { None };

            ui.painter().add(TextShape {
                fallback_color: GREEN,
                pos,
                galley: text_galley,
                override_text_color,
                underline: Stroke::NONE,
                angle: 0.,
            });
        }

        response
    }
}

// on_hover_text that doesn't cover things above or below, or the item itself, useful for comboboxes
pub fn on_hover_text_side(ui: UiRef, r: &Response, text: &str) {
    if !r.hovered() || text.is_empty() {
        return;
    }
    let description = shorten_string(text, 1000);
    let ctx = ui.ctx();
    let style = ui.style();
    let screen_rect = ctx.screen_rect();
    let right_space = ctx.screen_rect().right() - r.rect.right();
    let right = right_space > r.rect.left();
    let mut layout_job = WidgetText::RichText(color_text(&description, WHITE)).into_layout_job(
        ui.style(),
        FontSelection::Default,
        Align::Min,
    );
    layout_job.wrap.max_width = if right { right_space } else { r.rect.left() } - 30.;

    let galley = ui.fonts(|f| f.layout_job(layout_job));
    let size = galley.size() + style.spacing.menu_margin.sum();
    let y_offset = 'b: {
        let below = r.rect.top();
        if below + size.y < screen_rect.max.y {
            break 'b below;
        }
        let above = r.rect.bottom() - size.y;
        if above > 0. {
            break 'b above;
        }
        screen_rect.max.y / 2. - size.y / 2.
    };
    let x_offset = if right {
        r.rect.right() + 10.
    } else {
        r.rect.left() - 10. - size.x
    };

    let pos = screen_rect.left_top() + vec2(x_offset, y_offset);

    Area::new(ui.next_auto_id())
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
