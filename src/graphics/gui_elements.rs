use egui::{Align2, Context, Margin, Rect};
use std::fmt;
use std::time::{Duration, Instant};

use crate::events::input::world_to_screen;

use crate::graphics::camera::Camera;

#[derive(Clone, Debug)]
pub struct UiState {
    // pub viewport_rect: Option<egui::Rect>,
    // pub pixels_per_point: Option<f32>,
    pub ui_context: Option<Context>,
    pub theme: Theme,
    pub numeric_buff: String,
    // pub text_buff: String,
    pub text_edited: TextReplacement,
    pub numeric_active: bool,

    pub texts: Vec<Text>,
    pub action: Option<UiAction>,
    pub mode: UiMode,
    pub notifications: Vec<Notification>,
    pub cursor_position: Option<[f32; 2]>,
}

#[derive(Clone, Debug)]
pub struct Notification {
    message: String,
    created_at: Instant,
    ttl: Duration,
}

#[derive(Clone)]
pub struct Text {
    pub position: [f32; 2],
    pub contents: egui::WidgetText,
    pub rect: Option<egui::Rect>,
    pub editing: bool,
    pub annotative: bool,
}

#[derive(Clone, Debug)]
pub struct TextReplacement {
    pub contents: String,
    pub annotative: bool,
}

impl fmt::Debug for Text {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Text")
            .field("contents", &self.contents.text())
            .field("position", &self.position)
            .finish()
    }
}

#[derive(Clone, Debug)]
pub enum UiAction {
    DrawLine,
    DrawCircle,
    OpenFilePath(String),
    SaveFile,
    Input(String),
    TextEdited(TextReplacement),
    TextEditCancelled,
    ChangeTheme,
}

#[derive(Clone, Debug)]
pub enum UiMode {
    Normal,
    TextEdit,
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
    },
];

impl UiState {
    pub fn new() -> Self {
        let numeric_buff = String::new();
        // let text_buff = String::new();
        let theme = THEMES[0];
        let text_edited = TextReplacement {
            contents: String::new(),
            annotative: false,
        };

        Self {
            ui_context: None,
            theme,
            numeric_buff,
            text_edited,
            numeric_active: false,
            action: None,
            mode: UiMode::Normal,
            texts: Vec::new(),
            notifications: Vec::new(),
            cursor_position: None,
        }
    }

    pub fn viewport_rect(&self) -> Rect {
        self.ui_context
            .as_ref()
            .map(|ctx| ctx.available_rect())
            .unwrap_or(Rect::ZERO)
    }

    pub fn pixels_per_point(&self) -> f32 {
        self.ui_context
            .as_ref()
            .map(|ctx| ctx.pixels_per_point())
            .unwrap_or(1.0)
    }

    pub fn add_notification(&mut self, text: &str) {
        self.notifications.push(Notification {
            message: text.to_string(),
            created_at: Instant::now(),
            ttl: Duration::from_secs(5),
        });
    }

    pub fn push_digit(&mut self, ch: char) {
        self.numeric_active = true;
        self.numeric_buff.push(ch);
    }

    pub fn gui(&mut self, ui: &Context, camera: &mut Camera) {
        self.ui_context = Some(ui.clone());
        self.notifications
            .retain(|n| n.created_at.elapsed() < n.ttl);

        let pixels_per_point = ui.pixels_per_point();

        let viewport_rect = ui.available_rect();

        egui::Area::new(egui::Id::new("feature area"))
            .anchor(Align2::LEFT_TOP, [7.0, 5.0])
            .show(&ui, |ui| {
                let style = ui.style_mut();

                style.spacing.button_padding = egui::vec2(7.0, 4.0);
                style.text_styles.insert(
                    egui::TextStyle::Button,
                    egui::FontId::new(12.0, egui::FontFamily::Proportional),
                );

                style.visuals.widgets.inactive.weak_bg_fill = egui::Color32::from_rgb(40, 40, 40);
                style.visuals.widgets.inactive.bg_stroke =
                    egui::Stroke::new(1.0, egui::Color32::from_gray(80));
                style.visuals.widgets.inactive.fg_stroke =
                    egui::Stroke::new(1.0, egui::Color32::WHITE);
                style.visuals.widgets.inactive.rounding = egui::Rounding::same(3.0);

                style.visuals.widgets.hovered.weak_bg_fill = egui::Color32::from_rgb(45, 45, 45);
                style.visuals.widgets.hovered.rounding = egui::Rounding::same(3.0);

                style.visuals.widgets.active.weak_bg_fill = egui::Color32::from_rgb(45, 45, 45);
                style.visuals.widgets.active.expansion = 2.0;
                style.visuals.widgets.active.rounding = egui::Rounding::same(3.0);

                let painter = ui.painter();

                for text in &mut self.texts {
                    // println!("cursor: {:?}", cursor_pos);
                    let screen_position = world_to_screen(
                        text.position[0],
                        text.position[1],
                        viewport_rect,
                        camera,
                        pixels_per_point,
                    );
                    let rect = painter.text(
                        screen_position,
                        egui::Align2::LEFT_BOTTOM,
                        text.contents.text(),
                        egui::FontId::proportional(
                            14.0 * if text.annotative { 1.0 } else { camera.zoom },
                        ),
                        egui::Color32::WHITE,
                    );
                    text.rect = Some(rect);
                    // println!("{:?}", rect);
                }

                ui.horizontal(|ui| {
                    if ui.add(egui::Button::new("line")).clicked() {
                        self.action = Some(UiAction::DrawLine);
                    }

                    if ui.add(egui::Button::new("circle")).clicked() {
                        self.action = Some(UiAction::DrawCircle);
                    }

                    if ui.button("open").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_file() {
                            self.action = Some(UiAction::OpenFilePath(path.display().to_string()));
                        }
                    }

                    if ui.button("save").clicked() {
                        self.action = Some(UiAction::SaveFile);
                    }

                    if ui.button("toggle theme").clicked() {
                        if let Some(ind) = THEMES
                            .iter()
                            .position(|theme| theme.color_scheme == self.theme.color_scheme)
                        {
                            if ind == THEMES.len() - 1 {
                                self.theme = THEMES[0];
                            } else {
                                self.theme = THEMES[ind + 1];
                            }
                            self.action = Some(UiAction::ChangeTheme);
                        }
                    }

                    // if ui.button("add noti").clicked() {
                    //     self.add_notification("text");
                    // }
                });
            });

        // separate these two. put position into the corner, make notifications appear on top of them
        egui::Area::new(egui::Id::new("bottom right panel"))
            .anchor(Align2::RIGHT_BOTTOM, [0.0, 0.0])
            .show(&ui, |ui| {
                egui::Frame::none()
                    .inner_margin(egui::Margin {
                        right: 10.0,
                        bottom: 10.0,
                        ..Default::default()
                    })
                    .show(ui, |ui| {
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::Max), |ui| {
                            if let Some(cursor_pos) = self.cursor_position {
                                ui.label(format!("x: {:.3}", cursor_pos[0]));
                                ui.label(format!("y: {:.3}", cursor_pos[1]));
                            }
                        });
                    });
            });

        egui::Area::new(egui::Id::new("notifications"))
            .anchor(Align2::RIGHT_BOTTOM, [0.0, 0.0])
            .show(&ui, |ui| {
                egui::Frame::none()
                    .inner_margin(egui::Margin {
                        right: 10.0,
                        bottom: 27.0,
                        ..Default::default()
                    })
                    .show(ui, |ui| {
                        ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                            for note in &self.notifications {
                                let frame = egui::Frame::default()
                                    .fill(egui::Color32::from_rgb(40, 40, 40))
                                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(80)))
                                    .rounding(egui::Rounding::same(3.0))
                                    .multiply_with_opacity(0.5)
                                    .inner_margin(Margin::symmetric(7.0, 4.0));

                                frame.show(ui, |ui| {
                                    ui.set_max_width(250.0);
                                    ui.label(
                                        egui::RichText::new(&note.message)
                                            .color(egui::Color32::WHITE)
                                            .size(12.0),
                                    );
                                });
                                ui.add_space(5.0);
                            }
                        });
                    });
            });

        // text editing area
        if matches!(self.mode, UiMode::TextEdit) {
            // let mut text = self.texts.iter_mut().find(|t| t.editing).unwrap();
            egui::Window::new("Text Editor")
                .collapsible(false)
                .resizable(false)
                .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
                .show(&ui, |ui| {
                    ui.label("Modify your text:");
                    ui.text_edit_multiline(&mut self.text_edited.contents);
                    ui.checkbox(&mut self.text_edited.annotative, "Annotative");

                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            self.action = Some(UiAction::TextEditCancelled);
                            self.text_edited.contents.clear();
                            self.mode = UiMode::Normal;
                        }
                        if ui.button("Apply").clicked() {
                            self.action = Some(UiAction::TextEdited(self.text_edited.clone()));
                            self.text_edited.contents.clear();
                            self.mode = UiMode::Normal;
                        }
                    })
                });
        }

        // input palette
        egui::Area::new(egui::Id::new("input palette area"))
            .anchor(Align2::CENTER_BOTTOM, [0.0, 0.0])
            .show(&ui, |ui| {
                let style = ui.style_mut();

                style.visuals.extreme_bg_color = egui::Color32::from_rgb(40, 40, 40);
                style.visuals.widgets.inactive.bg_stroke =
                    egui::Stroke::new(1.0, egui::Color32::from_gray(80));
                style.visuals.widgets.inactive.rounding = egui::Rounding::same(3.0);
                style.visuals.widgets.hovered.rounding = egui::Rounding::same(3.0);
                style.visuals.widgets.active.rounding = egui::Rounding::same(3.0);

                ui.horizontal_centered(|ui| {
                    // let mut input = String::new();
                    let res = ui.add(
                        egui::TextEdit::singleline(&mut self.numeric_buff).desired_width(80.0),
                    );

                    if self.numeric_active {
                        res.request_focus();
                    }

                    // if focused
                    if res.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        println!("enter");
                        self.numeric_active = false;
                        let value = self.numeric_buff.clone();
                        self.action = Some(UiAction::Input(value));
                        self.numeric_buff.clear();
                        res.surrender_focus();
                    }
                })
            });
    }
}
