use crate::Mode;
use crate::State;
use egui::{Align2, Context};

// example
// pub fn GUI(ui: &Context) {
//     egui::Window::new("Streamline CFD")
//         .default_open(true)
//         .max_width(1000.0)
//         .max_height(800.0)
//         .default_width(800.0)
//         .resizable(true)
//         .anchor(Align2::LEFT_TOP, [0.0, 0.0])
//         .show(&ui, |mut ui| {
//             if ui.add(egui::Button::new("click me")).clicked() {
//                 println!("pressed");
//             }

//             ui.label("slider");

//             ui.end_row();
//         });
// }

pub fn GUI(ui: &Context, mut on_button_click: impl FnMut()) {
    egui::Area::new(egui::Id::new("idk"))
        .anchor(Align2::LEFT_TOP, [0.0, 0.0])
        .show(&ui, |ui| {
            if ui.add(egui::Button::new("line")).clicked() {
                println!("pressed");
                on_button_click();
            }
        });
}
