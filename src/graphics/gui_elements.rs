use egui::{Align2, Context};

#[derive(Clone, Debug)]
pub struct UiState {
    pub numeric_buff: String,
    pub numeric_active: bool,
    pub action: Option<UiAction>,
}

#[derive(Clone, Debug)]
pub enum UiAction {
    DrawLine,
    DrawCircle,
    OpenFilePath(String),
    SaveFile,
    Input(String),
}

impl UiState {
    pub fn new() -> Self {
        let numeric_buff = String::new();

        Self {
            numeric_buff,
            numeric_active: false,
            action: None,
        }
    }

    pub fn push_digit(&mut self, ch: char) {
        self.numeric_active = true;
        self.numeric_buff.push(ch);
    }

    pub fn gui(&mut self, ui: &Context) {
        egui::Area::new(egui::Id::new("feature area"))
            .anchor(Align2::LEFT_TOP, [7.0, 5.0])
            .show(&ui, |ui| {
                ui.horizontal(|ui| {
                    if ui.add(egui::Button::new("line")).clicked() {
                        self.action = Some(UiAction::DrawLine);
                    }
                    if ui.add(egui::Button::new("circle")).clicked() {
                        self.action = Some(UiAction::DrawCircle);
                    }

                    if ui.button("Open file…").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_file() {
                            self.action = Some(UiAction::OpenFilePath(path.display().to_string()));
                        }
                    }

                    if ui.button("Save").clicked() {
                        self.action = Some(UiAction::SaveFile);
                    }
                });
            });

        egui::Area::new(egui::Id::new("text palette area"))
            .anchor(Align2::CENTER_BOTTOM, [0.0, 0.0])
            .show(&ui, |ui| {
                ui.horizontal_centered(|ui| {
                    // let mut input = String::new();
                    let res = ui.add(egui::TextEdit::singleline(&mut self.numeric_buff).hint_text("command").desired_width(80.0));

                    if self.numeric_active {
                        res.request_focus();
                    }

                    // if focused
                    if res.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        println!("enter");
                        self.numeric_active = false;
                        self.action = Some(UiAction::Input(self.numeric_buff.clone()));
                        self.numeric_buff.clear();
                        res.surrender_focus();
                    }
                })
            });

    }
}