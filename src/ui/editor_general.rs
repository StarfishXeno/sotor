use egui::{ComboBox, CursorIcon, Frame, Grid, Label, Margin, RichText, Sense, Widget as _};

use crate::{
    save::{PartyMember, Save},
    util::format_seconds,
};

use super::{
    styles::{
        set_button_styles, set_button_styles_disabled, set_combobox_styles, set_selectable_styles,
        GREEN, GREY_DARK, WHITE,
    },
    widgets::UiExt as _,
    UiRef,
};

pub struct EditorGeneral<'a> {
    save: &'a mut Save,
    party_names: &'static [&'static str],
    selected_member: usize,
}
impl<'a> EditorGeneral<'a> {
    pub fn new(save: &'a mut Save) -> Self {
        Self {
            party_names: save.game.get_party_names(),
            save,
            selected_member: 0,
        }
    }
    pub fn show(&mut self, ui: UiRef) {
        egui::Grid::new("save_general")
            .num_columns(4)
            .spacing([20.0, 6.0])
            .show(ui, |ui| self.main_table(ui));

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

    fn main_table(&mut self, ui: UiRef) {
        let nfo = &mut self.save.nfo;
        let pt = &mut self.save.party_table;
        ui.label("Save name: ");
        ui.s_text_edit(&mut nfo.save_name);
        ui.label("Area name: ");
        ui.label(RichText::new(&nfo.area_name).color(WHITE));
        ui.end_row();

        ui.label("Time played: ");
        ui.label(RichText::new(format_seconds(nfo.time_played)).color(WHITE));
        ui.label("Cheats used: ");
        ui.s_checkbox(&mut nfo.cheats_used, "");
        ui.end_row();

        ui.label("Credits: ");
        ui.s_slider(&mut pt.credits, 0..=9_999_999);
        ui.label("Party XP: ");
        ui.s_slider(&mut pt.party_xp, 0..=9_999_999);
        ui.end_row();
    }

    fn member_table(&mut self, ui: UiRef) {
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

            ui.s_checkbox(&mut member.is_leader, "");

            ui.end_row();
        }
        if let Some(idx) = removed {
            members.remove(idx);
        }
    }

    fn party_member_selection(&mut self, ui: UiRef) {
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

        set_combobox_styles(ui);

        ComboBox::from_id_source("save_general_pt_member")
            .selected_text(self.party_names[self.selected_member])
            .show_ui(ui, |ui| {
                set_selectable_styles(ui);
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
        let mut btn = ui.s_button("Add", false, cant_add);

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
}
