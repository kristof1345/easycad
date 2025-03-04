use crate::Mode;
use egui::{Align2, Context};

pub fn GUI(ui: &Context) -> Option<Mode> {
    let mut mode = None;

    egui::Area::new(egui::Id::new("feature area"))
        .anchor(Align2::LEFT_TOP, [7.0, 5.0])
        .show(&ui, |ui| {
            if ui.add(egui::Button::new("line")).clicked() {
                mode = Some(Mode::DrawLine);
            }
            if ui.add(egui::Button::new("circle")).clicked() {
                mode = Some(Mode::DrawCircle);
            }
        });

    mode
}
