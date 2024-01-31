use super::{
    styles::{BLACK, GREEN, RED, WHITE},
    widgets::Icon,
    UiRef,
};
use egui::{
    text::{LayoutJob, LayoutSection},
    Color32, FontId, Frame, Margin, Response, RichText, TextFormat, TextStyle, WidgetText,
};
use egui_toast::{Toast, ToastKind, ToastOptions};

pub const SUCCESS_TOAST: u32 = 0;
pub const ERROR_TOAST: u32 = 1;

fn toast_frame(ui: UiRef, toast: &mut Toast, color: Color32, icon: RichText) -> Response {
    Frame::default()
        .fill(BLACK)
        .inner_margin(Margin::same(6.0))
        .rounding(4.0)
        .stroke((2.0, color))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(icon.text_style(TextStyle::Name("icon".into())).color(color));

                ui.label(toast.text.clone());
            });
        })
        .response
}

pub fn success_toast(ui: UiRef, toast: &mut Toast) -> Response {
    toast_frame(
        ui,
        toast,
        GREEN,
        RichText::new(Icon::Check.symbol()).size(20.),
    )
}

pub fn error_toast(ui: UiRef, toast: &mut Toast) -> Response {
    toast_frame(
        ui,
        toast,
        RED,
        RichText::new(Icon::Close.symbol()).size(30.),
    )
}

pub fn make_toast(text: String, content: Option<String>, success: bool) -> Toast {
    let text_len = text.len();
    let mut sections = vec![LayoutSection {
        leading_space: 0.,
        byte_range: 0..text_len,
        format: TextFormat::simple(FontId::default(), GREEN),
    }];
    let mut full_text = text;
    if let Some(str) = content {
        full_text += "\n";
        full_text += &str;
        sections.push(LayoutSection {
            leading_space: 0.,
            byte_range: text_len..full_text.len(),
            format: TextFormat::simple(FontId::default(), WHITE),
        });
    }
    let widget_text = WidgetText::LayoutJob(LayoutJob {
        sections,
        text: full_text,
        ..Default::default()
    });
    Toast {
        text: widget_text,
        kind: ToastKind::Custom(if success { SUCCESS_TOAST } else { ERROR_TOAST }),
        options: ToastOptions::default().duration_in_seconds(5.0),
    }
}
