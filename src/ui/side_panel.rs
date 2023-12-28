use crate::{
    ui::{styles::set_button_styles, widgets::UiExt, UiRef},
    util::{ContextExt, Message},
};

pub struct SidePanel {}

impl SidePanel {
    pub fn new() -> Self {
        Self {}
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
