use super::UiRef;

pub struct SidePanel {}

impl SidePanel {
    pub fn new() -> Self {
        Self {}
    }

    pub fn show(&mut self, ui: UiRef) {
        ui.separator();
    }
}
