use crate::{
    save::{AvailablePartyMember, Character, Nfo, PartyMember, PartyTable, Save},
    ui::{
        styles::{set_checkbox_styles, set_slider_styles, set_striped_styles, BLUE, GREEN, WHITE},
        widgets::{color_text, Icon, IconButton, UiExt},
        UiRef,
    },
    util::{format_seconds, ColumnCounter},
};
use egui::{Frame, Grid, Image, Layout, Margin, RichText, TextureHandle};

pub struct Editor<'a> {
    nfo: &'a mut Nfo,
    party_table: &'a mut PartyTable,
    characters: &'a mut [Character],
    image: &'a Option<TextureHandle>,
}

impl<'a> Editor<'a> {
    pub fn new(save: &'a mut Save) -> Self {
        Self {
            nfo: &mut save.nfo,
            party_table: &mut save.party_table,
            characters: &mut save.characters,
            image: &save.image,
        }
    }

    pub fn show(&mut self, ui: UiRef) {
        ui.horizontal_top(|ui| {
            egui::Grid::new("save_general")
                .num_columns(2)
                .spacing([20., 6.])
                .show(ui, |ui| self.main_table(ui));

            if self.image.is_some() {
                ui.with_layout(Layout::right_to_left(emath::Align::Min), |ui| {
                    Frame::default()
                        .stroke((4., GREEN))
                        .rounding(5.)
                        .outer_margin({
                            let mut margin = Margin::symmetric(0., 5.);
                            margin.left = 10.;
                            margin
                        })
                        .show(ui, |ui| self.image(ui));
                });
            }
        });

        ui.separator();

        set_striped_styles(ui);

        Grid::new("save_general_party")
            .num_columns(9)
            .min_col_width(0.)
            .spacing([10., 6.])
            .striped(true)
            .show(ui, |ui| {
                self.party_table(ui);
            });
    }

    fn image(&mut self, ui: UiRef) {
        let scale = 1.3;
        let image = Image::from((
            self.image.as_ref().unwrap().id(),
            (256. * scale, 144. * scale).into(),
        ))
        .rounding(5.);
        ui.add(image);
    }

    fn main_table(&mut self, ui: UiRef) {
        ui.label("Save name: ");
        ui.s_text_edit(&mut self.nfo.save_name, 200.);
        ui.end_row();

        ui.label("PC name: ");
        let name = self
            .nfo
            .pc_name
            .as_deref()
            .unwrap_or_else(|| &self.characters.first().unwrap().name);
        ui.s_text(name);
        ui.end_row();

        ui.label("Area name: ");
        ui.s_text(&self.nfo.area_name);
        ui.end_row();

        ui.label("Last module: ");
        ui.s_text(&self.nfo.last_module);
        ui.end_row();

        ui.label("Time played: ");
        ui.s_text(&format_seconds(self.nfo.time_played));
        ui.end_row();

        ui.label(color_text("Cheats used: ", GREEN));
        ui.horizontal(|ui| {
            set_checkbox_styles(ui);
            ui.s_checkbox(&mut self.nfo.cheat_used);
        });
        ui.end_row();

        set_slider_styles(ui);

        ui.label(color_text("Party XP: ", GREEN));
        ui.s_slider(&mut self.party_table.party_xp, 0..=9_999_999, true);
        ui.end_row();

        ui.label(color_text("Credits: ", GREEN));
        ui.s_slider(&mut self.party_table.credits, 0..=9_999_999, true);
        ui.end_row();

        if let Some(v) = &mut self.party_table.components {
            ui.label(color_text("Components: ", GREEN));
            ui.s_slider(v, 0..=99_999, true);
            ui.end_row();
        }

        if let Some(v) = &mut self.party_table.chemicals {
            ui.label(color_text("Chemicals: ", GREEN));
            ui.s_slider(v, 0..=99_999, true);
            ui.end_row();
        }
    }

    fn party_table(&mut self, ui: UiRef) {
        let members = &mut self.party_table.members;
        let available_members = &mut self.party_table.available_members;

        let columns = if self.characters.len() > 2 { 2 } else { 1 };

        for _ in 0..columns {
            ui.s_empty();
            ui.label(RichText::new("Name").underline());
            ui.label(RichText::new("Available").underline());
            ui.label(RichText::new("Selectable").underline());
        }
        ui.end_row();

        set_checkbox_styles(ui);
        let spacing = ui.spacing_mut();
        spacing.icon_width = 18.;
        spacing.icon_width_inner = 10.;

        let in_party: Vec<_> = members.iter().map(|m| m.idx).collect();

        let mut counter = ColumnCounter::new(2);
        for char in self.characters.iter().skip(1) {
            // PC taking a back seat
            if char.idx == usize::MAX - 1 {
                continue;
            }
            Self::member_row(
                ui,
                char.idx,
                &in_party,
                char,
                &mut available_members[char.idx],
                members,
            );
            counter.next(ui);
        }
    }

    fn member_row(
        ui: UiRef,
        idx: usize,
        in_party: &[usize],
        char: &Character,
        member: &mut AvailablePartyMember,
        members: &mut Vec<PartyMember>,
    ) {
        let in_party_idx = in_party
            .iter()
            .copied()
            .enumerate()
            .find(|(_, i)| *i == idx);

        if let Some((idx, _)) = in_party_idx {
            let btn = ui.s_icon_button(Icon::Leave, "Remove from current party");

            if btn.clicked() {
                members.remove(idx);
            }
        } else {
            let can_add = members.len() < 2 && member.available && member.selectable;

            let btn = ui.add_enabled(
                can_add,
                IconButton::new(Icon::Plus).hint("Add to current party"),
            );

            if btn.clicked() {
                members.push(PartyMember { idx, leader: false });
            }
        }

        let name = char.get_name();
        ui.label(if in_party_idx.is_some() {
            color_text(name, BLUE)
        } else {
            color_text(name, WHITE)
        });

        ui.s_checkbox(&mut member.available);
        ui.s_checkbox(&mut member.selectable);
    }
}
