use crate::{
    save::{PartyMember, Save},
    util::format_seconds,
};
use egui::{ComboBox, Frame, Grid, Image, Margin, RichText, TextureHandle};

use super::{
    styles::{
        set_button_styles, set_button_styles_disabled, set_combobox_styles, set_selectable_styles,
        GREEN, GREY_DARK,
    },
    widgets::UiExt,
    UiRef,
};

pub struct EditorGeneral<'a> {
    image: &'a TextureHandle,
    save: &'a mut Save,
    party_names: &'static [&'static str],
    selected_member: usize,
}
impl<'a> EditorGeneral<'a> {
    pub fn new(save: &'a mut Save, image: &'a TextureHandle) -> Self {
        Self {
            party_names: save.game.get_party_names(),
            image,
            save,
            selected_member: 0,
        }
    }
    pub fn show(&mut self, ui: UiRef) {
        ui.horizontal_top(|ui| {
            Frame::default()
                .stroke((4.0, GREEN))
                .rounding(5.0)
                .outer_margin({
                    let mut margin = Margin::ZERO;
                    margin.bottom = 5.0;
                    margin.right = 10.0;
                    margin
                })
                .show(ui, |ui| self.image(ui));

            egui::Grid::new("save_general")
                .num_columns(2)
                .spacing([20.0, 6.0])
                .show(ui, |ui| self.main_table(ui));
        });

        ui.separator();

        ui.horizontal_top(|ui| {
            ui.vertical(|ui| {
                ui.label("Current party: ");
                Self::party_grid(ui, "save_general_party", 3, |ui| self.member_table(ui));
                ui.horizontal(|ui| self.party_member_selection(ui));
            });
            ui.vertical(|ui| {
                ui.label("Available party: ");
                Self::party_grid(ui, "save_general_party_available", 7, |ui| {
                    self.available_table(ui);
                });
            })
        });
    }
    fn party_grid(ui: UiRef, id: &str, columns: usize, add_contents: impl FnOnce(UiRef)) {
        Frame::default()
            .stroke((2.0, GREEN))
            .inner_margin(10.0)
            .outer_margin({
                let mut margin = Margin::ZERO;
                margin.top = 4.0;
                margin.bottom = 8.0;
                margin
            })
            .rounding(5.0)
            .show(ui, |ui| {
                ui.visuals_mut().faint_bg_color = GREY_DARK;

                Grid::new(id)
                    .num_columns(columns)
                    .spacing([10.0, 6.0])
                    .show(ui, add_contents);
            });
    }
    fn image(&mut self, ui: UiRef) {
        let scale = ui.ctx().native_pixels_per_point().unwrap_or(1.0);
        let image =
            Image::from((self.image.id(), (256.0 * scale, 144.0 * scale).into())).rounding(5.0);
        ui.add(image);
    }

    fn main_table(&mut self, ui: UiRef) {
        let nfo = &mut self.save.nfo;
        let pt = &mut self.save.party_table;
        ui.label("Save name: ");
        ui.s_text_edit(&mut nfo.save_name);
        ui.end_row();

        ui.label("Area name: ");
        ui.s_text(&nfo.area_name);
        ui.end_row();

        ui.label("PC name: ");
        ui.s_text("TMP");
        ui.end_row();

        ui.label("Last module: ");
        ui.s_text(&nfo.last_module);
        ui.end_row();

        ui.label("Time played: ");
        ui.s_text(&format_seconds(nfo.time_played));
        ui.end_row();

        ui.label("Cheats used: ");
        ui.s_checkbox(&mut nfo.cheats_used, "");
        ui.end_row();

        ui.label("Credits: ");
        ui.s_slider(&mut pt.credits, 0..=9_999_999);
        ui.end_row();

        ui.label("Party XP: ");
        ui.s_slider(&mut pt.party_xp, 0..=9_999_999);
        ui.end_row();
    }

    fn member_table(&mut self, ui: UiRef) {
        let members = &mut self.save.party_table.members;

        ui.label(RichText::new("Name").underline());
        ui.label(RichText::new("Party leader").underline());
        ui.label("");
        ui.end_row();

        let mut removed = None;
        set_button_styles(ui);

        for (idx, member) in members.iter_mut().enumerate() {
            ui.s_text(self.party_names.get(member.idx).unwrap_or(&"UNKNOWN"));
            ui.s_checkbox(&mut member.leader, "");

            let btn = ui.s_button_basic("Remove");

            if btn.clicked() {
                removed = Some(idx);
            }
            ui.end_row();
        }
        if let Some(idx) = removed {
            members.remove(idx);
        }
    }

    fn party_member_selection(&mut self, ui: UiRef) {
        let pt = &mut self.save.party_table;
        let members = &mut pt.members;

        let available: Vec<_> = self
            .party_names
            .iter()
            .enumerate()
            .filter(|(idx, _)| {
                members.binary_search_by_key(idx, |m| m.idx).is_err()
                    && pt.available_members[*idx].available
                    && pt.available_members[*idx].selectable
            })
            .map(|(idx, name)| (idx, *name))
            .collect();

        if available.is_empty() {
            return;
        }

        set_combobox_styles(ui);

        ComboBox::from_id_source("save_general_pt_member")
            .selected_text(self.party_names[self.selected_member])
            .show_ui(ui, |ui| {
                set_selectable_styles(ui);
                for (idx, name) in available {
                    ui.selectable_value(&mut self.selected_member, idx, name);
                }
            });

        set_button_styles(ui);
        let cant_add = members.len() >= 2;
        if cant_add {
            set_button_styles_disabled(ui);
        }
        let mut btn = ui.s_button("Add", false, cant_add);

        if cant_add {
            btn = btn.on_hover_text("Can't have more than 2 members");
        }

        if !cant_add && btn.clicked() {
            members.push(PartyMember {
                idx: self.selected_member,
                leader: false,
            });
        }
    }

    fn available_table(&mut self, ui: UiRef) {
        let members = &mut self.save.party_table.available_members;

        ui.label(RichText::new("Name").underline());
        ui.label(RichText::new("Available").underline());
        ui.label(RichText::new("Selectable").underline());
        ui.label("");
        ui.label(RichText::new("Name").underline());
        ui.label(RichText::new("Available").underline());
        ui.label(RichText::new("Selectable").underline());
        ui.end_row();

        for (idx, members) in members.chunks_mut(2).enumerate() {
            ui.s_text(self.party_names.get(idx * 2).unwrap_or(&"UNKNOWN"));
            ui.s_checkbox(&mut members[0].available, "");
            ui.s_checkbox(&mut members[0].selectable, "");

            if members.len() > 1 {
                ui.label("");
                ui.s_text(self.party_names.get(idx * 2 + 1).unwrap_or(&"UNKNOWN"));
                ui.s_checkbox(&mut members[1].available, "");
                ui.s_checkbox(&mut members[1].selectable, "");
            }

            ui.end_row();
        }
    }
}
