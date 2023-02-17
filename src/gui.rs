use crate::settings::{ Settings, Toggleable };
use eframe::{ egui, egui::containers::ScrollArea, epaint };
use egui_extras::RetainedImage;
use poll_promise::Promise;
use std::{ collections::HashMap };
use sysinfo::{ PidExt, ProcessExt, System, SystemExt };

pub const APP_NAME: &str = r"Ragnarok - Maya Purple";
pub const LOGO_URL: &str = r"https://i.imgur.com/t3l5ncP.png";
pub const VERSION: &str = r"v1.0.0";

pub const HACK_CHECKBOX_LABEL: &str = r"Maya Purple";
pub const REFRESH_LABEL: &str = r"Refresh";
pub const PROCESS_NAME_LABEL: &str = r"Process name";
pub const FILTER_LABEL: &str = r"Filter";
pub struct CustomWindow {
    pid: u32,
    process_filter: String,
    system: System,
    processes: HashMap<u32, String>,
    maya_purple_hack: bool,
    settings: Settings,
    logo: Option<Promise<ehttp::Result<RetainedImage>>>,
}

pub fn draw_window() {
    let options = eframe::NativeOptions {
        resizable: false,
        decorated: false,
        initial_window_size: Some(egui::Vec2 { x: 300.0, y: 250.0 }),
        ..Default::default()
    };

    eframe::run_native(
        APP_NAME,
        options,
        Box::new(|ctx: &eframe::CreationContext| {
            let mut style = egui::Style::default();
            style.visuals.dark_mode = true;
            ctx.egui_ctx.set_style(style);

            Box::new(CustomWindow::default())
        })
    );
}

impl Default for CustomWindow {
    fn default() -> Self {
        let mut settings = Settings::new(false);

        let mut data = Self {
            pid: 0,
            process_filter: String::from(""),
            system: System::new_all(),
            processes: HashMap::new(),
            maya_purple_hack: false,
            settings: settings,
            logo: None,
        };

        data.system.refresh_all();

        for (pid, process) in data.system.processes() {
            data.processes.insert(pid.as_u32(), process.name().to_string());
        }

        data
    }
}

impl CustomWindow {}

impl eframe::App for CustomWindow {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let main_frame = egui::containers::Frame {
            rounding: egui::Rounding::none(),
            shadow: epaint::Shadow {
                extrusion: 0.0,
                color: egui::Color32::BLACK,
            },
            ..egui::containers::Frame::window(&egui::Style::default())
        };

        egui::TopBottomPanel
            ::top("top")
            .frame(main_frame)
            .show(ctx, |ui: &mut egui::Ui| {
                ui.horizontal(|ui: &mut egui::Ui| {
                    if ui.button("X").clicked() {
                        frame.close();
                    }

                    let item = egui::menu::bar(ui, |ui: &mut egui::Ui| {
                        ui.heading(APP_NAME);
                    });

                    if item.response.hovered() {
                        frame.drag_window();
                    }
                });
            });

        egui::TopBottomPanel
            ::bottom("bottom")
            .frame(main_frame)
            .show(ctx, |ui: &mut egui::Ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                    ui.small(VERSION);
                })
            });

        egui::CentralPanel
            ::default()
            .frame(main_frame)
            .show(ctx, |ui: &mut egui::Ui| {
                if self.pid != 0 {
                    let promise = self.logo.get_or_insert_with(|| {
                        let ctx = ctx.clone();
                        let (sender, promise) = Promise::new();
                        let request = ehttp::Request::get(LOGO_URL);

                        ehttp::fetch(request, move |response| {
                            let image = response.and_then(parse_response);
                            sender.send(image);
                            ctx.request_repaint();
                        });

                        promise
                    });

                    egui::CentralPanel::default().show(ctx, |ui| {
                        match promise.ready() {
                            None => {
                                ui.vertical_centered(|ui| {
                                    ui.spinner();
                                });
                            }
                            Some(Err(err)) => {
                                ui.vertical_centered(|ui| {
                                    ui.colored_label(ui.visuals().error_fg_color, err);
                                });
                            }
                            Some(Ok(image)) => {
                                ui.add_space(20.0);
                                ui.vertical_centered(|ui| {
                                    image.show_max_size(ui, ui.available_size());
                                });
                            }
                        }
                    });

                    ui.add_space(130.0);

                    ui.vertical_centered(|ui| {
                        if ui.checkbox(&mut self.maya_purple_hack, HACK_CHECKBOX_LABEL).clicked() {
                            self.settings.toggle_maya_purple_hack(self.pid);
                        }
                    });
                } else {
                    ui.horizontal(|ui: &mut egui::Ui| {
                        if ui.button(REFRESH_LABEL).clicked() {
                            self.system.refresh_all();
                            self.process_filter = String::from(PROCESS_NAME_LABEL);
                            self.processes = HashMap::new();
                            for (pid, process) in self.system.processes() {
                                self.processes.insert(pid.as_u32(), process.name().to_string());
                            }
                        }

                        if ui.button(FILTER_LABEL).clicked() {
                            self.system.refresh_all();

                            self.processes = HashMap::new();
                            for (pid, process) in self.system.processes() {
                                if
                                    process
                                        .name()
                                        .to_lowercase()
                                        .contains(&self.process_filter.to_lowercase())
                                {
                                    self.processes.insert(pid.as_u32(), process.name().to_string());
                                }
                            }
                        }

                        ui.text_edit_singleline(&mut self.process_filter);
                    });

                    ui.add_space(4.0);

                    ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .show_viewport(ui, |ui: &mut eframe::egui::Ui, _| {
                            let font_id = egui::TextStyle::Body.resolve(ui.style());
                            let row_height =
                                ui.fonts().row_height(&font_id) + ui.spacing().item_spacing.y;

                            ui.set_height((self.processes.len() as f32) * (row_height * 1.5));

                            for (pid, process) in &self.processes {
                                ui.horizontal(|ui| {
                                    ui.label(pid.to_string());

                                    if ui.link(process).clicked() {
                                        self.pid = *pid;
                                    }
                                });
                            }
                        });
                }
            });
    }
}

#[allow(clippy::needless_pass_by_value)]
fn parse_response(response: ehttp::Response) -> Result<RetainedImage, String> {
    let content_type = response.content_type().unwrap_or_default();

    if content_type.starts_with("image/") {
        RetainedImage::from_image_bytes(&response.url, &response.bytes)
    } else {
        Err(format!("Expected image, found content-type {:?}", content_type))
    }
}