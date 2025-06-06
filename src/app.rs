use std::io::{BufRead, BufReader};
use std::process::Stdio;
use std::sync::mpsc;

use egui::widgets::Slider;
use egui::{Align, CollapsingHeader, ComboBox, ProgressBar, RichText, TextStyle, Visuals};
use rfd::FileDialog;

use crate::depcheck::{can_run, exists};
use crate::encoding::{generate_command, parse_av1an_output};
use crate::models::{
    ColorPrimaries, ColorRange, MatrixCoefficients, PixelFormat, SourceLibrary, Theme,
    TransferCharacteristics,
};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct AV1Studio {
    pub av1an_verbosity_path: String,

    pub default_preset_path: String,

    #[serde(skip)]
    pub input_file: String,
    #[serde(skip)]
    pub output_file: String,
    #[serde(skip)]
    pub scenes_file: String,
    #[serde(skip)]
    pub zones_file: String,

    pub source_library: SourceLibrary,

    pub width: String,
    pub height: String,

    pub output_pixel_format: PixelFormat,
    pub color_primaries: ColorPrimaries,
    pub matrix_coefficients: MatrixCoefficients,
    pub transfer_characteristics: TransferCharacteristics,
    pub color_range: ColorRange,

    pub file_concatenation: String,

    pub preset: f32,
    pub crf: f32,
    pub synthetic_grain: String, // Synthetic grain is a String to allow editing
    pub custom_encode_params: String,

    #[serde(skip)]
    pub thread_affinity: String,
    #[serde(skip)]
    pub workers: String,

    #[serde(skip)]
    pub encoded_frames: Option<u32>,
    #[serde(skip)]
    pub total_frames: Option<u32>,
    #[serde(skip)]
    pub fps: Option<f64>,
    #[serde(skip)]
    pub eta_time: Option<String>,

    #[serde(skip)]
    pub encoding_in_progress: bool,
    #[serde(skip)]
    pub receiver: Option<mpsc::Receiver<String>>,

    #[serde(skip)]
    pub max_label_width: Option<f32>,
    #[serde(skip)]
    pub settings_max_label_width: Option<f32>,

    #[serde(skip)]
    pub show_settings_window: bool,

    pub active_theme: Theme,

    #[serde(skip)]
    pub av1an_verbosity_checked: bool,
    #[serde(skip)]
    pub av1an_verbosity_found: bool,

    #[serde(skip)]
    pub svtav1_checked: bool,
    #[serde(skip)]
    pub svtav1_found: bool,

    #[serde(skip)]
    pub show_av1an_verbosity_warning: bool,
    #[serde(skip)]
    pub show_svtav1_warning: bool,
}

impl Default for AV1Studio {
    fn default() -> Self {
        AV1Studio {
            av1an_verbosity_path: String::new(),
            default_preset_path: String::new(),
            input_file: String::new(),
            output_file: String::new(),
            scenes_file: String::new(),
            zones_file: String::new(),
            source_library: SourceLibrary::default(),
            width: String::from("1920"),
            height: String::from("1080"),
            output_pixel_format: PixelFormat::default(),
            color_primaries: ColorPrimaries::default(),
            matrix_coefficients: MatrixCoefficients::default(),
            transfer_characteristics: TransferCharacteristics::default(),
            color_range: ColorRange::default(),
            file_concatenation: String::new(),
            preset: 4.0,
            crf: 27.0,
            synthetic_grain: 0.to_string(),
            custom_encode_params: String::new(),
            thread_affinity: String::new(),
            workers: num_cpus::get_physical().to_string(),
            encoded_frames: None,
            total_frames: None,
            fps: None,
            eta_time: None,
            encoding_in_progress: false,
            receiver: None,
            max_label_width: None,
            settings_max_label_width: None,
            show_settings_window: false,
            active_theme: Theme::default(),
            av1an_verbosity_checked: false,
            av1an_verbosity_found: false,
            svtav1_checked: false,
            svtav1_found: false,
            show_av1an_verbosity_warning: false,
            show_svtav1_warning: false,
        }
    }
}

impl AV1Studio {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut style = (*cc.egui_ctx.style()).clone();
        style.text_styles.get_mut(&TextStyle::Body).unwrap().size = 18.0;
        style.text_styles.get_mut(&TextStyle::Heading).unwrap().size = 24.0;

        cc.egui_ctx.set_style(style);

        Self::default()
    }

    pub fn save_preset_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let preset = AV1StudioPreset {
            source_library: self.source_library.clone(),
            width: self.width.clone(),
            height: self.height.clone(),
            output_pixel_format: self.output_pixel_format.clone(),
            color_primaries: self.color_primaries.clone(),
            matrix_coefficients: self.matrix_coefficients.clone(),
            transfer_characteristics: self.transfer_characteristics.clone(),
            color_range: self.color_range.clone(),
            file_concatenation: self.file_concatenation.clone(),
            preset: self.preset,
            crf: self.crf,
            synthetic_grain: self.synthetic_grain.clone(),
            custom_encode_params: self.custom_encode_params.clone(),
        };

        let yaml = serde_yaml::to_string(&preset)?;
        std::fs::write(path, yaml)?;

        Ok(())
    }

    pub fn load_preset_from_file(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file_content = std::fs::read_to_string(path)?;
        let preset: AV1StudioPreset = serde_yaml::from_str(&file_content)?;

        self.source_library = preset.source_library;
        self.width = preset.width;
        self.height = preset.height;
        self.output_pixel_format = preset.output_pixel_format;
        self.color_primaries = preset.color_primaries;
        self.matrix_coefficients = preset.matrix_coefficients;
        self.transfer_characteristics = preset.transfer_characteristics;
        self.color_range = preset.color_range;
        self.file_concatenation = preset.file_concatenation;
        self.preset = preset.preset;
        self.crf = preset.crf;
        self.synthetic_grain = preset.synthetic_grain;
        self.custom_encode_params = preset.custom_encode_params;

        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
struct AV1StudioPreset {
    source_library: SourceLibrary,
    width: String,
    height: String,
    output_pixel_format: PixelFormat,
    color_primaries: ColorPrimaries,
    matrix_coefficients: MatrixCoefficients,
    transfer_characteristics: TransferCharacteristics,
    color_range: ColorRange,
    file_concatenation: String,
    preset: f32,
    crf: f32,
    synthetic_grain: String,
    custom_encode_params: String,
}

impl eframe::App for AV1Studio {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.max_label_width.is_none() {
            ctx.request_repaint();
            self.max_label_width = Some(0.0);
        }

        // Please tell me there's a better way to do this
        if !self.av1an_verbosity_checked {
            let path = std::path::Path::new("/usr/local/bin/av1an-verbosity");
            self.av1an_verbosity_found = exists(path);
            self.av1an_verbosity_checked = true;

            if !self.av1an_verbosity_found {
                self.show_av1an_verbosity_warning = true;
            }

            if !can_run(path) {
                eprintln!("WARNING: {:?} can't be found, you will have to give its path manually in the Settings menu", path);
            } else {
                println!("{:?} found", path);
            }
        }

        if !self.svtav1_checked {
            let path = std::path::Path::new("/usr/local/bin/SvtAv1EncApp");
            self.svtav1_found = exists(path);
            self.svtav1_checked = true;

            if !self.svtav1_found {
                self.show_svtav1_warning = true;
            }

            if !can_run(path) {
                eprintln!("WARNING: {:?} can't be found", path);
            } else {
                println!("{:?} found", path);
            }
        }

        if self.show_av1an_verbosity_warning {
            egui::Window::new("Warning")
                .open(&mut self.show_av1an_verbosity_warning)
                .show(ctx, |ui| {
                    ui.label("/usr/local/bin/av1an-verbosity not found! You will have to set a path for it manually in the Settings menu.");
                });
        }

        if self.show_svtav1_warning {
            egui::Window::new("Warning")
                .open(&mut self.show_svtav1_warning)
                .show(ctx, |ui| {
                    ui.label("/usr/local/bin/SvtAv1EncApp not found! You will have to set a path for it manually in the Settings menu.");
                });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("AV1Studio");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                    if ui.button("Settings").clicked() {
                        self.show_settings_window = true;
                    }
                    if self.show_settings_window {
                        egui::Window::new("Settings")
                            .open(&mut self.show_settings_window)
                            .show(ctx, |ui| {
                                let mut settings_max_label_width = self.settings_max_label_width.unwrap_or(0.0);
                                ui.label(RichText::new("Paths").weak());
                                ui.horizontal(|ui| {
                                    let label_text = "Av1an-verbosity Path";
                                    let label_width = ui.label(label_text).rect.max.x - ui.min_rect().min.x;
                                    settings_max_label_width = settings_max_label_width.max(label_width);
                                    if label_width < settings_max_label_width {
                                        ui.allocate_space(egui::vec2(settings_max_label_width - label_width, 1.0));
                                    } else {
                                        ui.allocate_space(egui::vec2(0.5, 1.0));
                                    }
                                    ui.add_sized(
                                        [500.0, 20.0],
                                        egui::TextEdit::singleline(&mut self.av1an_verbosity_path),
                                    );
                                    if ui.button("Browse").clicked() {
                                        if let Some(path) = FileDialog::new().pick_file() {
                                            self.av1an_verbosity_path = path.display().to_string();
                                        }
                                    }
                                    ui.label(RichText::new("ℹ").weak()).on_hover_ui(|ui| {
                                        ui.style_mut().interaction.selectable_labels = true;
                                        ui.label("Full path to the Av1an-verbosity binary.");
                                    });
                                });
                                ui.horizontal(|ui| {
                                    let label_text = "Default Preset Path";
                                    let label_width = ui.label(label_text).rect.max.x - ui.min_rect().min.x;
                                    settings_max_label_width = settings_max_label_width.max(label_width);
                                    if label_width < settings_max_label_width {
                                        ui.allocate_space(egui::vec2(settings_max_label_width - label_width, 1.0));
                                    }
                                    ui.add_sized(
                                        [500.0, 20.0],
                                        egui::TextEdit::singleline(&mut self.default_preset_path),
                                    );
                                    if ui.button("Browse").clicked() {
                                        if let Some(path) = FileDialog::new().pick_file() {
                                            self.av1an_verbosity_path = path.display().to_string();
                                        }
                                    }
                                    ui.label(RichText::new("ℹ").weak()).on_hover_ui(|ui| {
                                        ui.style_mut().interaction.selectable_labels = true;
                                        ui.label("Path to the YAML preset file that gets loaded every time AV1Studio is started.");
                                    });
                                });
                                ui.add_space(ui.spacing().item_spacing.y * 2.0);
                                ui.label(RichText::new("Looks").weak());
                                ui.horizontal(|ui| {
                                    let label_text = "Theme";
                                    let label_width = ui.label(label_text).rect.max.x - ui.min_rect().min.x;
                                    settings_max_label_width = settings_max_label_width.max(label_width);
                                    if label_width < settings_max_label_width {
                                        ui.allocate_space(egui::vec2(settings_max_label_width - label_width, 1.0));
                                    }
                                    ComboBox::from_id_salt("theme_switcher_combobox")
                                        .selected_text(self.active_theme.as_str())
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(
                                                &mut self.active_theme,
                                                Theme::Dark,
                                                "Dark",
                                            );
                                            ui.selectable_value(
                                                &mut self.active_theme,
                                                Theme::Light,
                                                "Light",
                                            );
                                        });
                                    ui.label(RichText::new("").weak()).on_hover_ui(|ui| {
                                        ui.style_mut().interaction.selectable_labels = true;
                                        ui.label("Name of the active theme.");
                                    });
                                });
                                ui.add_space(ui.spacing().item_spacing.y * 2.0);
                                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                                    if ui.button("Save").clicked() {
                                        if self.active_theme == Theme::Dark {
                                            ctx.set_visuals(Visuals::dark());
                                        } else if self.active_theme == Theme::Light {
                                            ctx.set_visuals(Visuals::light());
                                        }
                                    }
                                });
                            });
                    }
                    if ui.button("Load Preset").clicked() {
                        if let Some(path) = FileDialog::new()
                            .add_filter("YAML Files", &["yaml", "yml"])
                            .pick_file()
                        {
                            match self.load_preset_from_file(&path.display().to_string()) {
                                Ok(_) => {
                                    println!("Preset loaded successfully from {}", path.display());
                                },
                                Err(e) => {
                                    println!("Error loading preset: {}", e);
                                }
                            }
                        }
                    }
                    if ui.button("Save Preset").clicked() {
                        if let Some(path) = FileDialog::new()
                            .add_filter("YAML Files", &["yaml", "yml"])
                            .save_file()
                        {
                            let path_string = path.display().to_string();
                            let file_path = if path_string.ends_with(".yaml") || path_string.ends_with(".yml") {
                                path_string
                            } else {
                                format!("{}.yaml", path_string)
                            };

                            match self.save_preset_to_file(&file_path) {
                                Ok(_) => {
                                    println!("Preset saved successfully to {}", file_path);
                                },
                                Err(e) => {
                                    println!("Error saving preset: {}", e);
                                }
                            }
                        }
                    }
                });
            });
            ui.separator();

            // Wrap the main content in a ScrollArea
            egui::ScrollArea::vertical().show(ui, |ui| {
                let mut max_width = self.max_label_width.unwrap_or(0.0);

                CollapsingHeader::new(RichText::from("File Options").weak())
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let label_text = "*Input File";
                            let label_width = ui.label(label_text).rect.max.x - ui.min_rect().min.x;
                            max_width = max_width.max(label_width);
                            if label_width < max_width {
                                ui.allocate_space(egui::vec2(max_width - label_width, 1.0));
                            }
                            ui.add_sized(
                                [500.0, 20.0],
                                egui::TextEdit::singleline(&mut self.input_file),
                            );
                            if ui.button("Browse").clicked() {
                                if let Some(path) = FileDialog::new()
                                    .add_filter("Video Files", &[".mkv"])
                                    .pick_file()
                                {
                                    self.input_file = path.display().to_string();
                                }
                            }
                            ui.label(RichText::new("ℹ").weak()).on_hover_ui(|ui| {
                                ui.style_mut().interaction.selectable_labels = true;
                                ui.label("Full path to the input MKV file.");
                            });
                        });

                        ui.horizontal(|ui| {
                            let label_text = "*Output File";
                            let label_width = ui.label(label_text).rect.max.x - ui.min_rect().min.x;
                            max_width = max_width.max(label_width);
                            if label_width < max_width {
                                ui.allocate_space(egui::vec2(max_width - label_width, 1.0));
                            }
                            ui.add_sized(
                                [500.0, 20.0],
                                egui::TextEdit::singleline(&mut self.output_file),
                            );
                            if ui.button("Browse").clicked() {
                                if let Some(path) = FileDialog::new()
                                    .add_filter("Video Files", &["mkv"])
                                    .pick_file()
                                {
                                    self.output_file = path.display().to_string();
                                }
                            }
                            ui.label(RichText::new("ℹ").weak()).on_hover_ui(|ui| {
                                ui.style_mut().interaction.selectable_labels = true;
                                ui.label("Full path to the output MKV file.");
                            });
                        });

                        ui.horizontal(|ui| {
                            let label_text = "Scenes File";
                            let label_width = ui.label(label_text).rect.max.x - ui.min_rect().min.x;
                            max_width = max_width.max(label_width);
                            if label_width < max_width {
                                ui.allocate_space(egui::vec2(max_width - label_width, 1.0));
                            }
                            ui.add_sized(
                                [500.0, 20.0],
                                egui::TextEdit::singleline(&mut self.scenes_file),
                            );
                            if ui.button("Browse").clicked() {
                                if let Some(path) = FileDialog::new()
                                    .add_filter("JSON Files", &["json"])
                                    .pick_file()
                                {
                                    self.scenes_file = path.display().to_string();
                                }
                            }
                            ui.label(RichText::new("ℹ").weak()).on_hover_ui(|ui| {
                                ui.style_mut().interaction.selectable_labels = true;
                                ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                                    ui.label("Full path to a scenes file. (Check out");
                                    ui.hyperlink_to(
                                        RichText::new("Trix's Auto Boost Script")
                                            .color(egui::Color32::from_rgb(4, 165, 229)),
                                        "https://github.com/trixoniisama/auto-boost-algorithm",
                                    );
                                    ui.label(".)");
                                });
                            });
                        });

                        ui.horizontal(|ui| {
                            let label_text = "Zones File";
                            let label_width = ui.label(label_text).rect.max.x - ui.min_rect().min.x;
                            max_width = max_width.max(label_width);
                            if label_width < max_width {
                                ui.allocate_space(egui::vec2(max_width - label_width, 1.0));
                            }
                            ui.add_sized(
                                [500.0, 20.0],
                                egui::TextEdit::singleline(&mut self.zones_file),
                            );
                            if ui.button("Browse").clicked() {
                                if let Some(path) = FileDialog::new()
                                    .add_filter("TXT Files", &["txt"])
                                    .pick_file()
                                {
                                    self.zones_file = path.display().to_string();
                                }
                            }
                            ui.label(RichText::new("ℹ").weak()).on_hover_ui(|ui| {
                                ui.style_mut().interaction.selectable_labels = true;
                                ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                                    ui.label("Full path to a file specifying zones within the video with differing encoder settings. (Check out");
                                    ui.hyperlink_to(
                                        RichText::new("Trix's Auto Boost Script")
                                            .color(egui::Color32::from_rgb(4, 165, 229)),
                                        "https://github.com/trixoniisama/auto-boost-algorithm",
                                    );
                                    ui.label(".)");
                                });
                            });
                        });

                        ui.add_space(ui.spacing().item_spacing.y * 2.0);
                    });

                CollapsingHeader::new(RichText::from("Source Settings").weak())
                    .default_open(false)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let label_text = "*Source Library";
                            let label_width = ui.label(label_text).rect.max.x - ui.min_rect().min.x;
                            max_width = max_width.max(label_width);
                            if label_width < max_width {
                                ui.allocate_space(egui::vec2(max_width - label_width, 1.0));
                            }
                            ComboBox::from_id_salt("source_library_combobox")
                                .selected_text(self.source_library.as_str())
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut self.source_library,
                                        SourceLibrary::BestSource,
                                        "BestSource",
                                    );
                                    ui.selectable_value(
                                        &mut self.source_library,
                                        SourceLibrary::FFMS2,
                                        "FFMS2",
                                    );
                                    ui.selectable_value(
                                        &mut self.source_library,
                                        SourceLibrary::LSMASH,
                                        "L-SMASH",
                                    );
                                });
                            ui.label(RichText::new("ℹ").weak()).on_hover_ui(|ui| {
                                ui.style_mut().interaction.selectable_labels = true;
                                ui.label("Method to use for piping exact ranges of frames to the encoder (determines how frames are extracted and sent to the encoder). BestSource is now, supposedly, the best best and most accurate option, but slightly slower than L-SMASH and ffms2. L-SMASH can sometimes fuck up the frame orders completely. ffms2 might corrupt frames on problematic sources.");
                            });
                        });

                        ui.horizontal(|ui| {
                            let label_text = "File Concatenation";
                            let label_width = ui.label(label_text).rect.max.x - ui.min_rect().min.x;
                            max_width = max_width.max(label_width);
                            if label_width < max_width {
                                ui.allocate_space(egui::vec2(max_width - label_width, 1.0));
                            }
                            ui.add_sized(
                                [100.0, 20.0],
                                egui::TextEdit::singleline(&mut self.file_concatenation),
                            );
                            ui.label(RichText::new("ℹ").weak()).on_hover_ui(|ui| {
                                ui.style_mut().interaction.selectable_labels = true;
                                ui.label("Method to use for concatenating encoded chunks and audio into output file. If you don't know what you're doing, just go with the default option.");
                            });
                        });
                    });

                ui.add_space(ui.spacing().item_spacing.y * 2.0);

                CollapsingHeader::new(RichText::from("Video Settings").weak())
                    .default_open(false)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let label_text = "*(Output) Resolution";
                            let label_width = ui.label(label_text).rect.max.x - ui.min_rect().min.x;
                            max_width = max_width.max(label_width);
                            if label_width < max_width {
                                ui.allocate_space(egui::vec2(max_width - label_width, 1.0));
                            }
                            ui.add_sized(
                                [100.0, 20.0],
                                egui::TextEdit::singleline(&mut self.width),
                            );
                            ui.label("×");
                            ui.add_sized(
                                [100.0, 20.0],
                                egui::TextEdit::singleline(&mut self.height),
                            );
                            ui.label(RichText::new("ℹ").weak()).on_hover_ui(|ui| {
                                ui.style_mut().interaction.selectable_labels = true;
                                ui.label("Resolution to resize the output video to.");
                            });
                        });

                        ui.horizontal(|ui| {
                            let label_text = "*(Output) Pixel Format";
                            let label_width = ui.label(label_text).rect.max.x - ui.min_rect().min.x;
                            max_width = max_width.max(label_width);
                            if label_width < max_width {
                                ui.allocate_space(egui::vec2(max_width - label_width, 1.0));
                            }
                            ComboBox::from_id_salt("output_pixel_format_combobox")
                                .selected_text(self.output_pixel_format.as_str())
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut self.output_pixel_format,
                                        PixelFormat::Yuv420p10le,
                                        "yuv420p10le",
                                    );
                                    ui.selectable_value(
                                        &mut self.output_pixel_format,
                                        PixelFormat::Yuv420p,
                                        "yuv420p",
                                    );
                                });
                            ui.label(RichText::new("ℹ").weak()).on_hover_ui(|ui| {
                                ui.style_mut().interaction.selectable_labels = true;
                                ui.label("FFmpeg pixel format to use. It's best to go with yuv420p10le (10-bit color format), even if the input video has 8-bit colors.");
                            });
                        });

                        ui.horizontal(|ui| {
                            let label_text = "Color Primaries";
                            let label_width = ui.label(label_text).rect.max.x - ui.min_rect().min.x;
                            max_width = max_width.max(label_width);
                            if label_width < max_width {
                                ui.allocate_space(egui::vec2(max_width - label_width, 1.0));
                            }
                            ComboBox::from_id_salt("color_primaries_combobox")
                                .selected_text(self.color_primaries.as_str())
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut self.color_primaries,
                                        ColorPrimaries::Bt709,
                                        "(1) BT.709",
                                    );
                                    ui.selectable_value(
                                        &mut self.color_primaries,
                                        ColorPrimaries::Unspecified,
                                        "(2) Unspecified, Default",
                                    );
                                    ui.selectable_value(
                                        &mut self.color_primaries,
                                        ColorPrimaries::Bt470m,
                                        "(4) BT.470 System M (historical)",
                                    );
                                    ui.selectable_value(
                                        &mut self.color_primaries,
                                        ColorPrimaries::Bt470bg,
                                        "(5) BT.470 System B, G (historical)",
                                    );
                                    ui.selectable_value(
                                        &mut self.color_primaries,
                                        ColorPrimaries::Bt601,
                                        "(6) BT.601",
                                    );
                                    ui.selectable_value(
                                        &mut self.color_primaries,
                                        ColorPrimaries::Smpte240,
                                        "(7) SMPTE 240",
                                    );
                                    ui.selectable_value(
                                        &mut self.color_primaries,
                                        ColorPrimaries::Film,
                                        "(8) Generic Film (color filters using illuminant C)",
                                    );
                                    ui.selectable_value(
                                        &mut self.color_primaries,
                                        ColorPrimaries::Bt2020,
                                        "(9) BT.2020, BT.2100",
                                    );
                                    ui.selectable_value(
                                        &mut self.color_primaries,
                                        ColorPrimaries::Xyz,
                                        "(10) SMPTE 428 (CIE 1921 XYZ)",
                                    );
                                    ui.selectable_value(
                                        &mut self.color_primaries,
                                        ColorPrimaries::Smpte431,
                                        "(11) SMPTE RP 431-2",
                                    );
                                    ui.selectable_value(
                                        &mut self.color_primaries,
                                        ColorPrimaries::Smpte432,
                                        "(12) SMPT EG 432-1",
                                    );
                                    ui.selectable_value(
                                        &mut self.color_primaries,
                                        ColorPrimaries::Ebu3213,
                                        "(22) EBU Tech. 3213-E",
                                    );
                                });
                            ui.label(RichText::new("ℹ").weak()).on_hover_ui(|ui| {
                                ui.style_mut().interaction.selectable_labels = true;
                                ui.label("Color primaries, refer to the (SVT-AV1-PSY) user guide Appendix A.2 for full details. If you don't know what you're doing, just use the default option (2).");
                            });
                        });

                        ui.horizontal(|ui| {
                            let label_text = "Matrix Coefficients";
                            let label_width = ui.label(label_text).rect.max.x - ui.min_rect().min.x;
                            max_width = max_width.max(label_width);
                            if label_width < max_width {
                                ui.allocate_space(egui::vec2(max_width - label_width, 1.0));
                            }
                            ComboBox::from_id_salt("matrix_coefficients_combobox")
                                .selected_text(self.matrix_coefficients.as_str())
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut self.matrix_coefficients,
                                        MatrixCoefficients::Identity,
                                        "(0) Identity matrix",
                                    );
                                    ui.selectable_value(
                                        &mut self.matrix_coefficients,
                                        MatrixCoefficients::Bt709,
                                        "(1) BT.709",
                                    );
                                    ui.selectable_value(
                                        &mut self.matrix_coefficients,
                                        MatrixCoefficients::Unspecified,
                                        "(2) unspecified, default",
                                    );
                                    ui.selectable_value(
                                        &mut self.matrix_coefficients,
                                        MatrixCoefficients::Fcc,
                                        "(4) US FCC 73.628",
                                    );
                                    ui.selectable_value(
                                        &mut self.matrix_coefficients,
                                        MatrixCoefficients::Bt470bg,
                                        "(5) BT.470 System B, G (historical)",
                                    );
                                    ui.selectable_value(
                                        &mut self.matrix_coefficients,
                                        MatrixCoefficients::Bt601,
                                        "(6) BT.601",
                                    );
                                    ui.selectable_value(
                                        &mut self.matrix_coefficients,
                                        MatrixCoefficients::Smpte240,
                                        "(7) SMPTE 240 M",
                                    );
                                    ui.selectable_value(
                                        &mut self.matrix_coefficients,
                                        MatrixCoefficients::Ycgco,
                                        "(8) YCgCo",
                                    );
                                    ui.selectable_value(
                                        &mut self.matrix_coefficients,
                                        MatrixCoefficients::Bt2020Ncl,
                                        "(9) BT.2020 non-constant luminance, BT.2100 YCbCr",
                                    );
                                    ui.selectable_value(
                                        &mut self.matrix_coefficients,
                                        MatrixCoefficients::Bt2020Cl,
                                        "(10) BT.2020 constant luminance",
                                    );
                                    ui.selectable_value(
                                        &mut self.matrix_coefficients,
                                        MatrixCoefficients::Smpte2085,
                                        "(11) SMPTE ST 2085 YDzDx",
                                    );
                                    ui.selectable_value(
                                        &mut self.matrix_coefficients,
                                        MatrixCoefficients::ChromaNcl,
                                        "(12) Chromaticity-derived non-constant luminance",
                                    );
                                    ui.selectable_value(
                                        &mut self.matrix_coefficients,
                                        MatrixCoefficients::ChromaCl,
                                        "(13) Chromaticity-derived constant luminance",
                                    );
                                    ui.selectable_value(
                                        &mut self.matrix_coefficients,
                                        MatrixCoefficients::Ictcp,
                                        "(14) BT.2100 ICtCp",
                                    );
                                });
                            ui.label(RichText::new("ℹ").weak()).on_hover_ui(|ui| {
                                ui.style_mut().interaction.selectable_labels = true;
                                ui.label("Matrix coefficients, refer to the (SVT-AV1-PSY) user guide Appendix A.2 for full details. If you don't know what you're doing, just use the default option (2).");
                            });
                        });

                        ui.horizontal(|ui| {
                            let label_text = "Transfer Characteristics";
                            let label_width = ui.label(label_text).rect.max.x - ui.min_rect().min.x;
                            max_width = max_width.max(label_width);
                            if label_width < max_width {
                                ui.allocate_space(egui::vec2(max_width - label_width, 1.0));
                            }
                            ComboBox::from_id_salt("transfer_characteristics_combobox")
                                .selected_text(self.transfer_characteristics.as_str())
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut self.transfer_characteristics,
                                        TransferCharacteristics::Bt709,
                                        "(1) BT.709",
                                    );
                                    ui.selectable_value(
                                        &mut self.transfer_characteristics,
                                        TransferCharacteristics::Unpsecified,
                                        "(2) unspecified, default",
                                    );
                                    ui.selectable_value(
                                        &mut self.transfer_characteristics,
                                        TransferCharacteristics::Bt470m,
                                        "(4) BT.470 System M (historical)",
                                    );
                                    ui.selectable_value(
                                        &mut self.transfer_characteristics,
                                        TransferCharacteristics::Bt470bg,
                                        "(5) BT.470 System B, G (historical)",
                                    );
                                    ui.selectable_value(
                                        &mut self.transfer_characteristics,
                                        TransferCharacteristics::Bt601,
                                        "(6) BT.601",
                                    );
                                    ui.selectable_value(
                                        &mut self.transfer_characteristics,
                                        TransferCharacteristics::Smpte240,
                                        "(7) SMPTE 240 M",
                                    );
                                    ui.selectable_value(
                                        &mut self.transfer_characteristics,
                                        TransferCharacteristics::Linear,
                                        "(8) Linear",
                                    );
                                    ui.selectable_value(
                                        &mut self.transfer_characteristics,
                                        TransferCharacteristics::Log100,
                                        "(9) Logarithmic (100 : 1 range)",
                                    );
                                    ui.selectable_value(
                                        &mut self.transfer_characteristics,
                                        TransferCharacteristics::Log100Sqrt10,
                                        "(10) Logarithmic (100 * Sqrt(10) : 1 range)",
                                    );
                                    ui.selectable_value(
                                        &mut self.transfer_characteristics,
                                        TransferCharacteristics::Iec61966,
                                        "(11) IEC 61966-2-4",
                                    );
                                    ui.selectable_value(
                                        &mut self.transfer_characteristics,
                                        TransferCharacteristics::Bt1361,
                                        "(12) BT.1361",
                                    );
                                    ui.selectable_value(
                                        &mut self.transfer_characteristics,
                                        TransferCharacteristics::Srgb,
                                        "(13) sRGB or sYCC",
                                    );
                                    ui.selectable_value(
                                        &mut self.transfer_characteristics,
                                        TransferCharacteristics::Bt202010,
                                        "(14) BT.2020 10-bit systems",
                                    );
                                    ui.selectable_value(
                                        &mut self.transfer_characteristics,
                                        TransferCharacteristics::Bt202012,
                                        "(15) BT.2020 12-bit systems",
                                    );
                                    ui.selectable_value(
                                        &mut self.transfer_characteristics,
                                        TransferCharacteristics::Smpte2084,
                                        "(16) SMPTE ST 2084, ITU BT.2100 PQ",
                                    );
                                    ui.selectable_value(
                                        &mut self.transfer_characteristics,
                                        TransferCharacteristics::Smpte428,
                                        "(17) SMPTE ST 428",
                                    );
                                    ui.selectable_value(
                                        &mut self.transfer_characteristics,
                                        TransferCharacteristics::Hlg,
                                        "(18) BT.2100 HLG, ARIB STD-B67",
                                    );
                                });
                            ui.label(RichText::new("ℹ").weak()).on_hover_ui(|ui| {
                                ui.style_mut().interaction.selectable_labels = true;
                                ui.label("Transfer characteristics, refer to the user guide Appendix A.2 for full details. If you don't know what you're doing, just use the default option (2).");
                            });
                        });

                        ui.horizontal(|ui| {
                            let label_text = "Color Range";
                            let label_width = ui.label(label_text).rect.max.x - ui.min_rect().min.x;
                            max_width = max_width.max(label_width);
                            if label_width < max_width {
                                ui.allocate_space(egui::vec2(max_width - label_width, 1.0));
                            }
                            ComboBox::from_id_salt("color_range_combobox")
                                .selected_text(self.color_range.as_str())
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut self.color_range,
                                        ColorRange::Studio,
                                        "(0) studio, default",
                                    );
                                    ui.selectable_value(
                                        &mut self.color_range,
                                        ColorRange::Full,
                                        "(1) full",
                                    );
                                });
                            ui.label(RichText::new("ℹ").weak()).on_hover_ui(|ui| {
                                ui.style_mut().interaction.selectable_labels = true;
                                ui.label("Color range. If you don't know whast you're doing, just go with the default option (0).");
                            });
                        });
                    });

                ui.add_space(ui.spacing().item_spacing.y * 2.0);

                CollapsingHeader::new(RichText::from("Encoder Settings").weak())
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let label_text = "*Preset";
                            let label_width = ui.label(label_text).rect.max.x - ui.min_rect().min.x;
                            max_width = max_width.max(label_width);
                            if label_width < max_width {
                                ui.allocate_space(egui::vec2(max_width - label_width, 1.0));
                            }
                            ui.add(
                                Slider::new(&mut self.preset, 0.0..=13.0)
                                    .step_by(1.0)
                                    .custom_formatter(|n, _| format!("{}", n as i32)),
                            );
                            ui.label(RichText::new("ℹ").weak()).on_hover_ui(|ui| {
                                ui.style_mut().interaction.selectable_labels = true;
                                ui.label("Encoding preset to use. A very simple explanation is that you trade quality for encoding speed, the lower you go. Can be set from a range of 0-13. Generally, the sweet spot will be between 2-4-6, of course, depending on how powerful your CPU is, you might want to go higher.");
                            });
                        });

                        ui.horizontal(|ui| {
                            let label_text = "*CRF";
                            let label_width = ui.label(label_text).rect.max.x - ui.min_rect().min.x;
                            max_width = max_width.max(label_width);
                            if label_width < max_width {
                                ui.allocate_space(egui::vec2(max_width - label_width, 1.0));
                            }
                            ui.add(Slider::new(&mut self.crf, 0.0..=70.0).step_by(1.0));
                            ui.label(RichText::new("ℹ").weak()).on_hover_ui(|ui| {
                                ui.style_mut().interaction.selectable_labels = true;
                                ui.label("Sets CRF value. A simple explanation is that you trade file size for quality, the lower you go. Can be set from a range of 0-70, can be set in quarter steps (0.25). Generally, the sweet spot will be between 27-23.");
                            });
                        });

                        ui.horizontal(|ui| {
                            let label_text = "*Synthetic Grain";
                            let label_width = ui.label(label_text).rect.max.x - ui.min_rect().min.x;
                            max_width = max_width.max(label_width);
                            if label_width < max_width {
                                ui.allocate_space(egui::vec2(max_width - label_width, 1.0));
                            }
                            ui.add_sized(
                                [100.0, 20.0],
                                egui::TextEdit::singleline(&mut self.synthetic_grain),
                            );
                            ui.label(RichText::new("ℹ").weak()).on_hover_ui(|ui| {
                                ui.style_mut().interaction.selectable_labels = true;
                                ui.label("Sets the strength of the synthetic grain applied to the video.");
                            });
                        });

                        ui.horizontal(|ui| {
                            let label_text = "Custom Encoder Parameters";
                            let label_width = ui.label(label_text).rect.max.x - ui.min_rect().min.x;
                            max_width = max_width.max(label_width);
                            if label_width < max_width {
                                ui.allocate_space(egui::vec2(max_width - label_width, 1.0));
                            } else {
                                ui.allocate_space(egui::vec2(0.5, 1.0));
                            }
                            ui.add_sized(
                                [500.0, 20.0],
                                egui::TextEdit::singleline(&mut self.custom_encode_params),
                            );
                            ui.label(RichText::new("ℹ").weak()).on_hover_ui(|ui| {
                                ui.style_mut().interaction.selectable_labels = true;
                                ui.label("Provides SVT-AV1-PSY custom encoder parameters on top of the already included parameters.");
                            });
                        });
                    });

                ui.add_space(ui.spacing().item_spacing.y * 2.0);

                CollapsingHeader::new(RichText::from("Performance Settings").weak())
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let label_text = "*Thread Affinity";
                            let label_width = ui.label(label_text).rect.max.x - ui.min_rect().min.x;
                            max_width = max_width.max(label_width);
                            if label_width < max_width {
                                ui.allocate_space(egui::vec2(max_width - label_width, 1.0));
                            }
                            ui.add_sized(
                                [100.0, 20.0],
                                egui::TextEdit::singleline(&mut self.thread_affinity),
                            );
                            ui.label(RichText::new("ℹ").weak()).on_hover_ui(|ui| {
                                ui.style_mut().interaction.selectable_labels = true;
                                ui.label("Pin each worker to a specific set of threads of this size. Leaving this option unspecified allows the OS to schedule all processes spawned.");
                            });
                        });

                        ui.horizontal(|ui| {
                            let label_text = "*Workers";
                            let label_width = ui.label(label_text).rect.max.x - ui.min_rect().min.x;
                            max_width = max_width.max(label_width);
                            if label_width < max_width {
                                ui.allocate_space(egui::vec2(max_width - label_width, 1.0));
                            }
                            ui.add_sized(
                                [100.0, 20.0],
                                egui::TextEdit::singleline(&mut self.workers),
                            );
                            ui.label(RichText::new("ℹ").weak()).on_hover_ui(|ui| {
                                ui.style_mut().interaction.selectable_labels = true;
                                ui.label("Number of workers to spawn. It's generally recommended, if you have enough RAM, to set this to the total amount of CPU cores you have for better encoding speeds. Leaving this at the default value will allow Av1an to figure out the amount of workers to spawn automatically.");
                            });
                        });
                    });

                self.max_label_width = Some(max_width);
            });

            // Pin the Start Encoding section to the bottom
            ui.with_layout(egui::Layout::bottom_up(Align::Center), |ui| {
                if self.encoding_in_progress {
                    if let Some(receiver) = &self.receiver {
                        loop {
                            match receiver.try_recv() {
                                Ok(line) => {
                                    println!("Received from channel: {}", line);
                                    parse_av1an_output(
                                        &line,
                                        &mut self.encoded_frames,
                                        &mut self.total_frames,
                                        &mut self.fps,
                                        &mut self.eta_time,
                                    )
                                }
                                Err(mpsc::TryRecvError::Empty) => break,
                                Err(mpsc::TryRecvError::Disconnected) => {
                                    self.encoding_in_progress = false;
                                    self.receiver = None;
                                    break;
                                }
                            }
                        }
                    }
                }

                let (ef, tf) = (
                    self.encoded_frames.unwrap_or_default(),
                    self.total_frames.unwrap_or_default(),
                );
                let progress = if tf == 0 { 0.0 } else { ef as f32 / tf as f32 };
                ui.add(ProgressBar::new(progress).show_percentage());

                ui.horizontal(|ui| {
                    ui.label("Encoded frames | Total frames:");
                    ui.label(&format!("{} | {}", ef, tf));
                });

                if ui.button("Start Encoding").clicked() {
                    let mut cmd = generate_command(self);
                    println!("{:?}", cmd);
                    let (sender, receiver) = mpsc::channel();
                    self.receiver = Some(receiver);
                    self.encoding_in_progress = true;

                    std::thread::spawn(move || {
                        let mut child = cmd
                            .stdout(Stdio::piped())
                            .stderr(Stdio::piped())
                            .spawn()
                            .expect("failed to start av1an");

                        let stdout = child.stdout.take().unwrap();
                        let stderr = child.stderr.take().unwrap();
                        let sender_stdout = sender.clone();
                        let sender_stderr = sender.clone();

                        std::thread::spawn(move || {
                            let reader = BufReader::new(stdout);
                            for line in reader.lines() {
                                if let Ok(line) = line {
                                    sender_stdout.send(line).unwrap();
                                }
                            }
                        });

                        std::thread::spawn(move || {
                            let reader = BufReader::new(stderr);
                            for line in reader.lines() {
                                if let Ok(line) = line {
                                    sender_stderr.send(line).unwrap();
                                }
                            }
                        });

                        let _ = child.wait();
                    });
                }

                ctx.request_repaint();
            });
        });
    }
}
