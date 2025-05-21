use egui::{Align2, Context};

#[derive(Clone, Debug)]
pub enum UiAction {
    DrawLine,
    DrawCircle,
    OpenFilePath(String),
    SaveFile,
}

pub fn GUI(ui: &Context) -> Option<UiAction> {
    let mut action = None;

    egui::Area::new(egui::Id::new("feature area"))
        .anchor(Align2::LEFT_TOP, [7.0, 5.0])
        .show(&ui, |ui| {
            ui.horizontal(|ui| {
                if ui.add(egui::Button::new("line")).clicked() {
                    action = Some(UiAction::DrawLine);
                }
                if ui.add(egui::Button::new("circle")).clicked() {
                    action = Some(UiAction::DrawCircle);
                }

                if ui.button("Open file…").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                        action = Some(UiAction::OpenFilePath(path.display().to_string()));
                    }
                }

                if ui.button("Save").clicked() {
                    action = Some(UiAction::SaveFile);
                }
            });
        });

    action
}
