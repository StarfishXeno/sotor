use std::collections::HashMap;

use crate::{
    save::Game,
    ui::{styles::set_button_styles, widgets::UiExt, UiRef},
    util::{ContextExt, Directory, Message},
};

pub struct SidePanel<'a> {
    save_list: &'a [HashMap<String, Vec<Directory>>; Game::GAME_COUNT],
}

impl<'a> SidePanel<'a> {
    pub fn new(save_list: &'a [HashMap<String, Vec<Directory>>; Game::GAME_COUNT]) -> Self {
        Self { save_list }
    }

    pub fn show(&mut self, ui: UiRef) {
        ui.s_offset([0.0, 3.0]);
        set_button_styles(ui);
        let btn = ui.s_button_basic("Settings");
        if btn.clicked() {
            let channel = ui.ctx().get_channel();
            channel.send(Message::OpenSettings).unwrap();
        }
        ui.separator();
    }
}
