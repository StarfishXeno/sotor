use egui::{
    style::{Selection, Spacing, WidgetVisuals, Widgets},
    Color32, Context, FontData, FontDefinitions,
    FontFamily::{Name, Proportional},
    FontId, TextStyle, Visuals,
};

use super::UiRef;

pub const WHITE: Color32 = Color32::from_rgb(223, 223, 201);
pub const GREEN: Color32 = Color32::from_rgb(47, 192, 161);
pub const GREEN_DARK: Color32 = Color32::from_rgb(29, 119, 99);
pub const BLUE: Color32 = Color32::from_rgb(0, 168, 249);
pub const RED: Color32 = Color32::from_rgb(183, 26, 0);
pub const GREY: Color32 = Color32::from_rgb(131, 131, 106);
pub const GREY_DARK: Color32 = Color32::from_rgb(52, 52, 52);
pub const BLACK: Color32 = Color32::from_rgb(26, 26, 26);
pub const BLACK_TRANSPARENT: Color32 = Color32::from_rgba_premultiplied(0, 0, 0, 100);

pub fn set_selectable_styles(ui: UiRef) {
    let visuals = ui.visuals_mut();
    visuals.override_text_color = Some(WHITE);
    visuals.selection.bg_fill = GREY_DARK;
    visuals.widgets.hovered.weak_bg_fill = GREY_DARK;
}

pub fn set_combobox_styles(ui: UiRef) {
    let styles = ui.style_mut();
    let visuals = &mut styles.visuals;
    let spacing = &mut styles.spacing;

    spacing.button_padding = [4., 1.].into();
    spacing.icon_width = 16.;

    visuals.override_text_color = Some(BLACK);
}

pub fn set_striped_styles(ui: UiRef) {
    ui.visuals_mut().faint_bg_color = GREY_DARK;
}

pub fn set_drag_value_styles(ui: UiRef) {
    let styles = ui.style_mut();
    let visuals = &mut styles.visuals;
    let widgets = &mut visuals.widgets;

    styles.spacing.interact_size = [20., 20.].into();
    visuals.override_text_color = Some(BLACK);

    widgets.inactive.weak_bg_fill = WHITE;
    widgets.hovered.weak_bg_fill = WHITE;
    widgets.active.weak_bg_fill = WHITE;
    widgets.active.bg_stroke = (2., GREEN).into();
}
pub fn set_slider_styles(ui: UiRef) {
    set_drag_value_styles(ui);
}
pub fn set_checkbox_styles(ui: UiRef) {
    let visuals = ui.visuals_mut();
    let widgets = &mut visuals.widgets;
    let stroke = (3., BLACK).into();

    visuals.override_text_color = Some(BLACK);

    widgets.hovered.fg_stroke = stroke;
    widgets.active.fg_stroke = stroke;
    widgets.inactive.fg_stroke = stroke;
    widgets.active.bg_stroke = (2., GREEN).into();
}

pub fn set_button_styles(ui: UiRef) {
    let visuals = ui.visuals_mut();
    let widgets = &mut visuals.widgets;

    visuals.override_text_color = Some(GREEN);

    widgets.inactive.weak_bg_fill = BLACK;
    widgets.hovered.weak_bg_fill = BLACK;
    widgets.active.weak_bg_fill = BLACK;
    widgets.active.bg_stroke = (2., WHITE).into();
}

#[allow(dead_code)]
pub fn set_button_styles_disabled(ui: UiRef) {
    let visuals = ui.visuals_mut();
    let widgets = &mut visuals.widgets;
    let stroke = (2., GREY).into();

    visuals.override_text_color = Some(GREY);

    widgets.inactive.bg_stroke = stroke;
    widgets.hovered.bg_stroke = stroke;
    widgets.active.bg_stroke = stroke;
}

pub fn make_fonts() -> FontDefinitions {
    const FONT_NAME: &str = "Roboto-Medium";
    const FONT_FA_SOLID_NAME: &str = "fa-solid-900";
    let mut fonts = FontDefinitions::default();

    fonts.font_data.insert(
        FONT_NAME.to_owned(),
        FontData::from_static(include_bytes!("../../assets/Roboto-Medium.ttf")),
    );
    fonts
        .families
        .get_mut(&Proportional)
        .unwrap()
        .insert(0, FONT_NAME.into());
    // ICONS
    fonts.font_data.insert(
        FONT_FA_SOLID_NAME.to_owned(),
        FontData::from_static(include_bytes!("../../assets/fa-solid-900.ttf")),
    );
    fonts
        .families
        .insert(Name("Icons".into()), vec![FONT_FA_SOLID_NAME.into()]);

    fonts
}

pub fn set_styles(ctx: &Context) {
    let mut style = (*ctx.style()).clone();
    ctx.set_fonts(make_fonts());
    style.text_styles = [
        (
            TextStyle::Name("icon".into()),
            FontId::new(22., Name("Icons".into())),
        ),
        (TextStyle::Heading, FontId::new(22., Proportional)),
        (
            TextStyle::Name("large".into()),
            FontId::new(16., Proportional),
        ),
        (TextStyle::Body, FontId::new(14., Proportional)),
        (TextStyle::Monospace, FontId::new(14., Proportional)),
        (TextStyle::Button, FontId::new(14., Proportional)),
        (TextStyle::Small, FontId::new(13., Proportional)),
    ]
    .into();
    style.spacing = Spacing {
        icon_width: 20.,
        icon_width_inner: 11.,
        ..style.spacing
    };
    style.drag_value_text_style = TextStyle::Body;

    style.visuals = Visuals {
        text_cursor: (2., GREY).into(),
        override_text_color: Some(GREEN),
        hyperlink_color: BLUE,
        faint_bg_color: WHITE,
        extreme_bg_color: WHITE,
        window_fill: BLACK,
        window_stroke: (1., GREEN_DARK).into(),
        panel_fill: BLACK,

        selection: Selection {
            bg_fill: BLACK,
            stroke: (2., GREEN).into(),
        },

        widgets: Widgets {
            noninteractive: make_widget_visuals(GREY),
            open: make_widget_visuals(GREEN),
            hovered: make_widget_visuals(GREEN),
            active: make_widget_visuals(GREEN),
            inactive: make_widget_visuals(GREEN_DARK),
        },
        ..style.visuals
    };
    ctx.set_style(style);
}

fn make_widget_visuals(stroke_color: Color32) -> WidgetVisuals {
    WidgetVisuals {
        bg_fill: WHITE,
        weak_bg_fill: WHITE,
        bg_stroke: (2., stroke_color).into(),
        rounding: 2.0.into(),
        fg_stroke: (1., stroke_color).into(),
        expansion: 0.,
    }
}
