use std::ops::RangeInclusive;

use crate::{
    save::{Game, PartyMember, PartyTable, Save},
    util::format_seconds,
};
use egui::{
    Button, ComboBox, CursorIcon, Frame, Grid, Label, Layout, Response, RichText, Sense, Slider,
    TextBuffer, Ui, Vec2, Widget,
};
use emath::{Numeric, Rect};
use sotor_macros::EnumToString;

use self::styles::{BLACK, DARK_GREY, GREEN, GREY, WHITE};

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
    save: Save,
    tab: Tab,
}

impl Default for SotorApp {
    fn default() -> Self {
        let save = Save::read_from_directory("./save/", Game::One).unwrap();
        Self {
            party_names: get_party_names(&save.game),
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
        ui.style_mut().visuals.override_text_color = Some(BLACK);
        ui.add(
            Slider::new(value, range)
                .logarithmic(true)
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
fn button(ui: &mut Ui, text: &str, selected: bool) -> Response {
    let mut text = RichText::new(text);
    if selected {
        text = text.color(WHITE);
    }
    let mut btn = Button::new(text).selected(selected).rounding(2.0);
    if selected {
        btn = btn.stroke((2.0, WHITE));
    }
    btn.ui(ui).on_hover_cursor(CursorIcon::PointingHand)
}
fn checkbox(ui: &mut Ui, value: &mut bool, text: &str) {
    ui.horizontal(|ui| {
        let stroke = (2.0, BLACK).into();
        ui.style_mut().visuals.widgets.hovered.fg_stroke = stroke;
        ui.style_mut().visuals.widgets.active.fg_stroke = stroke;
        ui.style_mut().visuals.widgets.inactive.fg_stroke = stroke;
        ui.checkbox(value, text)
            .on_hover_cursor(CursorIcon::PointingHand);
    });
}
const fn get_party_names(game: &Game) -> &'static [&'static str] {
    const PARTY_1: &[&str] = &[
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

    const PARTY_2: &[&str] = &[
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
        let member_indices: Vec<_> = pt.party.iter().map(|m| m.idx).collect();

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
            .rounding(5.0)
            .show(ui, |ui| {
                ui.visuals_mut().faint_bg_color = DARK_GREY;

                Grid::new("save_general_party")
                    .num_columns(2)
                    .spacing([10.0, 6.0])
                    .show(ui, |ui| Self::fill_member_table(ui, pt, self.party_names));
            });

        let mut member_to_add = None;
        ui.horizontal(|ui| {
            let available: Vec<_> = self
                .party_names
                .into_iter()
                .enumerate()
                .filter(|(idx, _)| member_indices.contains(idx))
                .collect();

            let mut selected = available[0].0;
            ComboBox::new("save_general_pt_member", "")
                .selected_text(self.party_names[selected])
                .show_ui(ui, |ui| {
                    for (idx, name) in available {
                        ui.selectable_value(&mut selected, idx, *name);
                    }
                });
            set_button_styles(ui);
            let btn = button(ui, "Add", false);
            if btn.clicked() {
                member_to_add = Some(PartyMember {
                    idx: selected,
                    is_leader: false,
                });
            }
        });
        if let Some(member) = member_to_add {
            pt.party.push(member)
        }
    }
    fn fill_member_table(ui: &mut Ui, pt: &mut PartyTable, names: &[&str]) {
        let party = &mut pt.party;
        ui.label(RichText::new("Name").underline());
        ui.label(RichText::new("Party leader").underline());
        ui.end_row();

        let mut removed = None;
        for (idx, member) in party.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                let btn = Label::new(RichText::new("🚷").size(16.0).color(WHITE))
                    .sense(Sense::click())
                    .ui(ui)
                    .on_hover_cursor(CursorIcon::PointingHand);

                if btn.clicked() {
                    removed = Some(idx);
                }
                ui.label(RichText::new(*names.get(member.idx).unwrap_or(&"UNKNOWN")).color(WHITE));
            });

            checkbox(ui, &mut member.is_leader, "");

            ui.end_row();
        }
        if let Some(idx) = removed {
            party.remove(idx);
        }
    }
    fn show_globals(&mut self, ui: &mut Ui) {}
}

impl eframe::App for SotorApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("save_tabs").show(ctx, |ui| {
            ui.horizontal(|ui| {
                set_button_styles(ui);
                for tab in Tab::UNIT_VALUES {
                    let btn = button(ui, &tab.to_string(), self.tab == tab);
                    if btn.clicked() {
                        self.tab = tab;
                    }
                }
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| match self.tab {
            Tab::General => self.show_general(ui),
            Tab::Globals => self.show_globals(ui),
            _ => {}
        });
    }
}
