use crate::{
    save::{AvailablePartyMember, Game, PartyMember, Save},
    ui::{
        styles::{
            set_button_styles, set_button_styles_disabled, set_checkbox_styles, set_striped_styles,
            BLUE, GREEN,
        },
        widgets::{white_text, UiExt},
        UiRef,
    },
    util::{format_seconds, ColumnCounter},
};
use egui::{Frame, Grid, Image, Layout, Margin, RichText};

pub struct EditorGeneral<'a> {
    save: &'a mut Save,
    party_names: &'static [&'static str],
}

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

impl<'a> EditorGeneral<'a> {
    pub fn new(save: &'a mut Save) -> Self {
        let party_names = match save.game {
            Game::One => PARTY_1,
            Game::Two => PARTY_2,
        };

        Self { save, party_names }
    }
    pub fn show(&mut self, ui: UiRef) {
        ui.horizontal_top(|ui| {
            egui::Grid::new("save_general")
                .num_columns(2)
                .spacing([20.0, 6.0])
                .show(ui, |ui| self.main_table(ui));

            ui.with_layout(Layout::right_to_left(emath::Align::Min), |ui| {
                Frame::default()
                    .stroke((4.0, GREEN))
                    .rounding(5.0)
                    .outer_margin({
                        let mut margin = Margin::symmetric(0.0, 5.0);
                        margin.left = 10.0;
                        margin
                    })
                    .show(ui, |ui| self.image(ui));
            })
        });

        ui.separator();

        set_striped_styles(ui);

        Grid::new("save_general_party")
            .num_columns(9)
            .spacing([10.0, 6.0])
            .striped(true)
            .show(ui, |ui| {
                self.party_table(ui);
            });

        ui.s_offset([0.0, 2.0]);
        ui.label(RichText::new("*currently in your party").color(BLUE));
    }

    fn image(&mut self, ui: UiRef) {
        let scale = 1.3;
        let image = Image::from((self.save.image.id(), (256.0 * scale, 144.0 * scale).into()))
            .rounding(5.0);
        ui.add(image);
    }

    fn main_table(&mut self, ui: UiRef) {
        let nfo = &mut self.save.nfo;
        let pt = &mut self.save.party_table;
        ui.label("Save name: ");
        ui.s_text_edit(&mut nfo.save_name, 200.0);
        ui.end_row();

        ui.label("PC name: ");
        ui.s_text("TMP");
        ui.end_row();

        ui.label("Area name: ");
        ui.s_text(&nfo.area_name);
        ui.end_row();

        ui.label("Last module: ");
        ui.s_text(&nfo.last_module);
        ui.end_row();

        ui.label("Time played: ");
        ui.s_text(&format_seconds(nfo.time_played));
        ui.end_row();

        ui.label("Cheats used: ");
        ui.s_checkbox(&mut nfo.cheats_used);
        ui.end_row();

        ui.label("Party XP: ");
        ui.s_slider(&mut pt.party_xp, 0..=9_999_999);
        ui.end_row();

        ui.label("Credits: ");
        ui.s_slider(&mut pt.credits, 0..=9_999_999);
        ui.end_row();

        if self.save.game == Game::Two {
            ui.label("Components: ");
            ui.s_slider(&mut pt.components, 0..=99_999);
            ui.end_row();

            ui.label("Chemicals: ");
            ui.s_slider(&mut pt.chemicals, 0..=99_999);
            ui.end_row();
        }
    }

    fn party_table(&mut self, ui: UiRef) {
        let members = &mut self.save.party_table.members;
        let available_members = &mut self.save.party_table.available_members;

        ui.label(RichText::new("Name").underline());
        ui.label(RichText::new("Available").underline());
        ui.label(RichText::new("Selectable").underline());
        ui.label("");
        ui.label("");
        ui.label(RichText::new("Name").underline());
        ui.label(RichText::new("Available").underline());
        ui.label(RichText::new("Selectable").underline());
        ui.label("");
        ui.end_row();

        set_checkbox_styles(ui);
        let spacing = ui.spacing_mut();
        spacing.icon_width = 18.0;
        spacing.icon_width_inner = 10.0;

        let in_party: Vec<_> = members.iter().map(|m| m.idx).collect();

        let mut counter = ColumnCounter::new(2);
        for (idx, member) in available_members.iter_mut().enumerate() {
            Self::member_row(ui, idx, &in_party, self.party_names, member, members);
            counter.next(ui);
        }
    }

    fn member_row(
        ui: UiRef,
        idx: usize,
        in_party: &[usize],
        party_names: &[&str],
        member: &mut AvailablePartyMember,
        members: &mut Vec<PartyMember>,
    ) {
        let in_party_idx = in_party
            .iter()
            .copied()
            .enumerate()
            .find(|(_, i)| *i == idx);
        let name = party_names.get(idx).unwrap_or(&"UNKNOWN");

        ui.label(if in_party_idx.is_some() {
            RichText::new(*name).color(BLUE)
        } else {
            white_text(name)
        });
        ui.add_enabled_ui(false, |ui| ui.s_checkbox_raw(&mut member.available));
        ui.s_checkbox_raw(&mut member.selectable);

        ui.horizontal(|ui| {
            set_button_styles(ui);
            if let Some((idx, _)) = in_party_idx {
                let btn = ui.s_button_basic("Remove");

                if btn.clicked() {
                    members.remove(idx);
                }
            } else {
                ui.style_mut().spacing.button_padding = [17.2, 1.0].into();
                let cant_add = members.len() >= 2 || !member.available || !member.selectable;
                if cant_add {
                    set_button_styles_disabled(ui);
                }
                let btn = ui.s_button("Add", false, cant_add);

                if !cant_add && btn.clicked() {
                    members.push(PartyMember { idx, leader: false });
                }
            }
        });
    }
}
