use std::ops::RangeInclusive;

use crate::{
    save::{Game, PartyMember, PartyTable, Save},
    util::format_seconds,
};
use egui::{
    style::HandleShape, Button, ComboBox, CursorIcon, Frame, Grid, Label, Margin, Response,
    RichText, Sense, Slider, SliderOrientation, TextBuffer, Ui, Vec2, Widget,
};
use emath::Numeric;
use sotor_macros::EnumToString;

use self::styles::{BLACK, GREEN, GREY, GREY_DARK, WHITE};

mod styles;

#[derive(EnumToString, PartialEq, Clone, Copy)]
enum Tab {
    General,
    Globals,
    Characters,
    Inventory,
    Quests,
}

pub struct SotorApp {
    party_names: &'static [&'static str],
    selected_member: usize,
    save: Save,
    tab: Tab,
}

impl Default for SotorApp {
    fn default() -> Self {
        let save = Save::read_from_directory("./save/", Game::One).unwrap();
        Self {
            party_names: get_party_names(&save.game),
            selected_member: 0,
            save,
            tab: Tab::General,
        }
    }
}

fn text_edit(ui: &mut Ui, text: &mut dyn TextBuffer, enabled: bool) -> Response {
    ui.add_enabled(
        enabled,
        egui::TextEdit::singleline(text)
            .vertical_align(egui::Align::Center)
            .min_size(Vec2::new(300.0, 0.0))
            .text_color(BLACK),
    )
}
fn slider<T: Numeric>(ui: &mut Ui, value: &mut T, range: RangeInclusive<T>) {
    ui.horizontal(|ui| {
        ui.visuals_mut().override_text_color = Some(BLACK);
        ui.add(
            Slider::new(value, range)
                .logarithmic(true)
                .handle_shape(HandleShape::Rect { aspect_ratio: 1.0 })
                .clamp_to_range(true),
        )
    });
}
fn set_button_styles(ui: &mut Ui) {
    let widgets = &mut ui.visuals_mut().widgets;
    widgets.inactive.weak_bg_fill = BLACK;
    widgets.hovered.weak_bg_fill = BLACK;
    widgets.active.weak_bg_fill = BLACK;
    widgets.active.bg_stroke = (2.0, WHITE).into();
}

fn set_button_styles_disabled(ui: &mut Ui) {
    let visuals = ui.visuals_mut();
    let widgets = &mut visuals.widgets;
    let stroke = (2.0, GREY).into();
    visuals.override_text_color = Some(GREY);
    widgets.inactive.bg_stroke = stroke;
    widgets.hovered.bg_stroke = stroke;
    widgets.active.bg_stroke = stroke;
}
fn button(ui: &mut Ui, text: &str, selected: bool, disabled: bool) -> Response {
    let mut text = RichText::new(text);
    if selected {
        text = text.color(WHITE);
    }
    let mut btn = Button::new(text).selected(selected).rounding(2.0);
    if selected {
        btn = btn.stroke((2.0, WHITE));
    }
    btn.ui(ui).on_hover_cursor(if disabled {
        CursorIcon::NotAllowed
    } else {
        CursorIcon::PointingHand
    })
}
fn checkbox(ui: &mut Ui, value: &mut bool, text: &str) {
    ui.horizontal(|ui| {
        let widgets = &mut ui.visuals_mut().widgets;
        let stroke = (2.0, BLACK).into();
        widgets.hovered.fg_stroke = stroke;
        widgets.active.fg_stroke = stroke;
        widgets.inactive.fg_stroke = stroke;
        ui.checkbox(value, text)
            .on_hover_cursor(CursorIcon::PointingHand);
    });
}
fn get_party_names(game: &Game) -> &'static [&'static str] {
    static PARTY_1: &[&str] = &[
        "Bastila",
        "Canderous",
        "Carth",
        "HK-47",
        "Jolee",
        "Juhani",
        "Mission",
        "T3-M4",
        "Zaalbar",
    ];

    static PARTY_2: &[&str] = &[
        "Atton",
        "Bao-Dur",
        "Mandalore",
        "G0-T0",
        "Handmaiden",
        "HK-47",
        "Kreia",
        "Mira",
        "T3-M4",
        "Visas",
        "Hanharr",
        "Disciple",
    ];

    match game {
        Game::One => PARTY_1,
        Game::Two => PARTY_2,
    }
}

impl SotorApp {
    pub fn new(ctx: &eframe::CreationContext<'_>) -> Self {
        styles::set_style(&ctx.egui_ctx);
        SotorApp::default()
    }

    fn show_general(&mut self, ui: &mut Ui) {
        let nfo = &mut self.save.nfo;
        let pt = &mut self.save.party_table;

        egui::Grid::new("save_general")
            .num_columns(4)
            .spacing([20.0, 6.0])
            .show(ui, |ui| {
                ui.label("Save name: ");
                text_edit(ui, &mut nfo.save_name, true);
                ui.label("Area name: ");
                ui.label(RichText::new(&nfo.area_name).color(WHITE));
                ui.end_row();

                ui.label("Time played: ");
                ui.label(RichText::new(format_seconds(nfo.time_played)).color(WHITE));
                ui.label("Cheats used: ");
                checkbox(ui, &mut nfo.cheats_used, "");
                ui.end_row();

                ui.label("Credits: ");
                slider(ui, &mut pt.credits, 0..=9_999_999);
                ui.label("Party XP: ");
                slider(ui, &mut pt.party_xp, 0..=9_999_999);
                ui.end_row();
            });

        ui.separator();

        ui.label("Current party: ");
        Frame::default()
            .stroke((2.0, GREEN))
            .inner_margin(10.0)
            .outer_margin({
                let mut margin = Margin::ZERO;
                margin.bottom = 5.0;
                margin
            })
            .rounding(5.0)
            .show(ui, |ui| {
                ui.visuals_mut().faint_bg_color = GREY_DARK;

                Grid::new("save_general_party")
                    .num_columns(2)
                    .spacing([10.0, 6.0])
                    .show(ui, |ui| self.member_table(ui));
            });

        ui.horizontal(|ui| self.party_member_selection(ui));
    }
    fn member_table(&mut self, ui: &mut Ui) {
        let members = &mut self.save.party_table.members;

        ui.label(RichText::new("Name").underline());
        ui.label(RichText::new("Party leader").underline());
        ui.end_row();

        let mut removed = None;
        for (idx, member) in members.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                let btn = Label::new(RichText::new("🚷").size(16.0).color(WHITE))
                    .sense(Sense::click())
                    .ui(ui)
                    .on_hover_cursor(CursorIcon::PointingHand)
                    .on_hover_text("Remove from party");

                if btn.clicked() {
                    removed = Some(idx);
                }
                ui.label(
                    RichText::new(*self.party_names.get(member.idx).unwrap_or(&"UNKNOWN"))
                        .color(WHITE),
                );
            });

            checkbox(ui, &mut member.is_leader, "");

            ui.end_row();
        }
        if let Some(idx) = removed {
            members.remove(idx);
        }
    }

    fn party_member_selection(&mut self, ui: &mut Ui) {
        let members = &mut self.save.party_table.members;
        let available: Vec<_> = self
            .party_names
            .iter()
            .enumerate()
            .filter(|(idx, _)| members.binary_search_by_key(idx, |m| m.idx).is_err())
            .map(|(idx, name)| (idx, *name))
            .collect();

        if available.is_empty() {
            return;
        }

        let visuals = ui.visuals_mut();
        visuals.override_text_color = Some(BLACK);

        ComboBox::from_id_source("save_general_pt_member")
            .selected_text(self.party_names[self.selected_member])
            .show_ui(ui, |ui| {
                let visuals = ui.visuals_mut();
                visuals.override_text_color = Some(WHITE);
                visuals.selection.bg_fill = GREY_DARK;
                visuals.widgets.hovered.weak_bg_fill = GREY_DARK;

                for (idx, name) in available {
                    ui.selectable_value(&mut self.selected_member, idx, name);
                }
            });

        let visuals = ui.visuals_mut();
        visuals.override_text_color = Some(GREEN);

        set_button_styles(ui);
        let cant_add = members.len() >= 2;
        if cant_add {
            set_button_styles_disabled(ui);
        }
        let mut btn = button(ui, "Add", false, cant_add);

        if cant_add {
            btn = btn.on_hover_text("Can't have more than 2 members");
        }

        if btn.clicked() {
            members.push(PartyMember {
                idx: self.selected_member,
                is_leader: false,
            });
        }
    }
    fn show_globals(&mut self, ui: &mut Ui) {}
}

impl eframe::App for SotorApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("save_tabs")
            .show_separator_line(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    set_button_styles(ui);
                    for tab in Tab::UNIT_VALUES {
                        let btn = button(ui, &tab.to_string(), self.tab == tab, false);
                        if btn.clicked() {
                            self.tab = tab;
                        }
                    }
                });
                ui.separator();
            });
        egui::CentralPanel::default().show(ctx, |ui| match self.tab {
            Tab::General => self.show_general(ui),
            Tab::Globals => self.show_globals(ui),
            _ => {}
        });
    }
}
