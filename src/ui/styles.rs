use egui::{
    style::{Selection, Spacing, WidgetVisuals, Widgets},
    Color32, Context, FontData, FontDefinitions,
    FontFamily::{self, Proportional},
    FontId, TextStyle, Ui, Visuals,
};

pub const WHITE: Color32 = Color32::from_rgb(223, 223, 201);
pub const GREEN: Color32 = Color32::from_rgb(47, 192, 161);
pub const GREEN_DARK: Color32 = Color32::from_rgb(29, 119, 99);
pub const BLUE: Color32 = Color32::from_rgb(199, 255, 250);
pub const RED: Color32 = Color32::from_rgb(183, 26, 0);
pub const GREY: Color32 = Color32::from_rgb(131, 131, 106);
pub const GREY_DARK: Color32 = Color32::from_rgb(52, 52, 52);
pub const BLACK: Color32 = Color32::from_rgb(26, 26, 26);

pub fn set_selectable_styles(ui: &mut Ui) {
    let visuals = ui.visuals_mut();
    visuals.override_text_color = Some(WHITE);
    visuals.selection.bg_fill = GREY_DARK;
    visuals.widgets.hovered.weak_bg_fill = GREY_DARK;
}
pub fn set_combobox_styles(ui: &mut Ui) {
    let visuals = ui.visuals_mut();
    visuals.override_text_color = Some(BLACK);
}
pub fn set_slider_styles(ui: &mut Ui) {
    let visuals = ui.visuals_mut();
    visuals.override_text_color = Some(BLACK);
}
pub fn set_checkbox_styles(ui: &mut Ui) {
    let widgets = &mut ui.visuals_mut().widgets;
    let stroke = (2.0, BLACK).into();

    widgets.hovered.fg_stroke = stroke;
    widgets.active.fg_stroke = stroke;
    widgets.inactive.fg_stroke = stroke;
    widgets.active.bg_stroke = (2.0, GREEN).into();
}

pub fn set_button_styles(ui: &mut Ui) {
    let visuals = ui.visuals_mut();
    let widgets = &mut visuals.widgets;

    visuals.override_text_color = Some(GREEN);

    widgets.inactive.weak_bg_fill = BLACK;
    widgets.hovered.weak_bg_fill = BLACK;
    widgets.active.weak_bg_fill = BLACK;
    widgets.active.bg_stroke = (2.0, WHITE).into();
}

pub fn set_button_styles_disabled(ui: &mut Ui) {
    let visuals = ui.visuals_mut();
    let widgets = &mut visuals.widgets;
    let stroke = (2.0, GREY).into();

    visuals.override_text_color = Some(GREY);

    widgets.inactive.bg_stroke = stroke;
    widgets.hovered.bg_stroke = stroke;
    widgets.active.bg_stroke = stroke;
}

pub fn make_fonts() -> FontDefinitions {
    const FONT_NAME: &str = "Roboto-Medium";

    let mut fonts = FontDefinitions::default();

    fonts.font_data.insert(
        FONT_NAME.to_owned(),
        FontData::from_static(include_bytes!("../../assets/Roboto-Medium.ttf")),
    );

    fonts
        .families
        .insert(FontFamily::Name(FONT_NAME.into()), vec![FONT_NAME.into()]);

    fonts
        .families
        .get_mut(&FontFamily::Proportional)
        .unwrap() //it works
        .insert(0, FONT_NAME.into());

    fonts
        .families
        .get_mut(&FontFamily::Monospace)
        .unwrap()
        .insert(0, FONT_NAME.into());

    fonts
}

pub fn set_styles(ctx: &Context) {
    let mut style = (*ctx.style()).clone();
    ctx.set_fonts(make_fonts());
    style.text_styles = [
        (TextStyle::Heading, FontId::new(22.0, Proportional)),
        (TextStyle::Body, FontId::new(14.0, Proportional)),
        (TextStyle::Monospace, FontId::new(14.0, Proportional)),
        (TextStyle::Button, FontId::new(14.0, Proportional)),
        (TextStyle::Small, FontId::new(12.0, Proportional)),
    ]
    .into();
    style.spacing = Spacing { ..style.spacing };

    style.visuals = Visuals {
        text_cursor: (2.0, GREY).into(),
        override_text_color: Some(GREEN),
        hyperlink_color: BLUE,
        faint_bg_color: WHITE,
        extreme_bg_color: WHITE,
        window_fill: BLACK,
        panel_fill: BLACK,

        selection: Selection {
            bg_fill: BLACK,
            stroke: (2.0, GREEN).into(),
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
        bg_stroke: (2.0, stroke_color).into(),
        rounding: 2.0.into(),
        fg_stroke: (1.0, stroke_color).into(),
        expansion: 0.0,
    }
}
