use egui::{Align2, Context};

#[derive(Clone, Debug)]
pub struct UiState {
    pub theme: Theme,
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
    ChangeTheme,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ColorScheme {
    Light,
    Grey,
    Dark,
}

#[derive(Copy, Clone, Debug)]
pub struct Theme {
    pub color_scheme: ColorScheme,
    pub colors: [f64; 3], 
}

const THEMES: [Theme; 3] = [
    Theme {
        color_scheme: ColorScheme::Dark,
        colors: [1.0, 1.0, 1.0],
    },
    Theme {
        color_scheme: ColorScheme::Grey,
        colors: [5.0, 8.0, 12.0],
    },
    Theme {
        color_scheme: ColorScheme::Light,
        colors: [255.0, 255.0, 255.0],
    }
];

impl UiState {
    pub fn new() -> Self {
        let numeric_buff = String::new();

        // let bg_color = [5.0, 8.0, 12.0];
        let theme = THEMES[0];
        // let bg_color = [255.0, 255.0, 255.0];

        Self {
            theme,
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

                    if ui.button("open file…").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_file() {
                            self.action = Some(UiAction::OpenFilePath(path.display().to_string()));
                        }
                    }

                    if ui.button("save").clicked() {
                        self.action = Some(UiAction::SaveFile);
                    }

                    if ui.button("toggle theme").clicked() {
                        if let Some(ind) = THEMES.iter().position(|theme| theme.color_scheme == self.theme.color_scheme) {
                            if ind == THEMES.len() - 1 {
                                self.theme = THEMES[0];
                            } else {
                                self.theme = THEMES[ind + 1];
                            }
                            self.action = Some(UiAction::ChangeTheme);
                        }
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