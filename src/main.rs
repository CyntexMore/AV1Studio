use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::mpsc;

use egui::widgets::Slider;
use egui::{ComboBox, ProgressBar, RichText, TextStyle};
use regex::Regex;

use rfd::FileDialog;

use catppuccin_egui::{set_theme, MOCHA};

struct AV1Studio {
    av1an_verbosity_path: String,

    input_file: String,
    output_file: String,
    scenes_file: String,
    zones_file: String,

    source_library: SourceLibrary,

    width: String,
    height: String,

    output_pixel_format: PixelFormat,
    color_primaries: ColorPrimaries,
    matrix_coefficients: MatrixCoefficients,

    file_concatenation: String,

    preset: f32,
    crf: f32,
    synthetic_grain: String, // Synthetic grain is a String to allow editing
    custom_encode_params: String,

    thread_affinity: String,
    workers: String,

    encoded_frames: Option<u32>,
    total_frames: Option<u32>,
    fps: Option<f64>,
    eta_time: Option<String>,

    encoding_in_progress: bool,
    receiver: Option<mpsc::Receiver<String>>,

    max_label_width: Option<f32>,
}

#[derive(PartialEq, Eq, Clone, Copy, Default)]
enum SourceLibrary {
    #[default]
    BestSource,
    FFMS2,
    LSMASH,
}

impl Default for AV1Studio {
    fn default() -> Self {
        AV1Studio {
            av1an_verbosity_path: String::new(),
            input_file: String::new(),
            output_file: String::new(),
            scenes_file: String::new(),
            zones_file: String::new(),
            source_library: SourceLibrary::default(),
            width: String::new(),
            height: String::new(),
            output_pixel_format: PixelFormat::default(),
            color_primaries: ColorPrimaries::default(),
            matrix_coefficients: MatrixCoefficients::default(),
            file_concatenation: String::new(),
            preset: 4.0,
            crf: 27.0,
            synthetic_grain: 0.to_string(),
            custom_encode_params: String::new(),
            thread_affinity: String::new(),
            workers: 0.to_string(),
            encoded_frames: None,
            total_frames: None,
            fps: None,
            eta_time: None,
            encoding_in_progress: false,
            receiver: None,
            max_label_width: None,
        }
    }
}

impl SourceLibrary {
    fn as_str(&self) -> &str {
        match self {
            SourceLibrary::BestSource => "BestSource",
            SourceLibrary::FFMS2 => "FFMS2",
            SourceLibrary::LSMASH => "L-SMASH",
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum PixelFormat {
    Yuv420p,
    Yuv420p10le,
}

impl Default for PixelFormat {
    fn default() -> Self {
        PixelFormat::Yuv420p10le
    }
}

impl PixelFormat {
    fn as_str(&self) -> &str {
        match self {
            PixelFormat::Yuv420p => "yuv420p",
            PixelFormat::Yuv420p10le => "yuv420p10le",
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum ColorPrimaries {
    Bt709,       // [1] BT.709
    Unspecified, // [2] unspecified, default
    Bt470m,      // [4] BT.470 System M (historical)
    Bt470bg,     // [5] BT.470 System B, G (historical)
    Bt601,       // [6] BT.601
    Smpte240,    // [7] SMPTE 240
    Film,        // [8] Generic film (color filters using illuminant C)
    Bt2020,      // [9] SMPTE 428 (CIE 1921 XYZ)
    Xyz,         // [10] SMPTE RP 431-2
    Smpte431,    // [11] SMPTE EG 431-2
    Smpte432,    // [12] SMPTE EG 432-1
    Ebu3213,     // [22] EBU Tech. 3213-E
}

impl Default for ColorPrimaries {
    fn default() -> Self {
        ColorPrimaries::Unspecified
    }
}

impl ColorPrimaries {
    fn as_str(&self) -> &str {
        match self {
            ColorPrimaries::Bt709 => "1",
            ColorPrimaries::Unspecified => "2",
            ColorPrimaries::Bt470m => "4",
            ColorPrimaries::Bt470bg => "5",
            ColorPrimaries::Bt601 => "6",
            ColorPrimaries::Smpte240 => "7",
            ColorPrimaries::Film => "8",
            ColorPrimaries::Bt2020 => "9",
            ColorPrimaries::Xyz => "10",
            ColorPrimaries::Smpte431 => "11",
            ColorPrimaries::Smpte432 => "12",
            ColorPrimaries::Ebu3213 => "22",
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum MatrixCoefficients {
    Identity,    // [0] Identity matrix
    Bt709,       // [1] BT.709
    Unspecified, // [2] unspecified, default
    Fcc,         // [4] US FCC 73.628
    Bt470bg,     // [5] BT.470 System B, G (historical)
    Bt601,       // [6] BT.601
    Smpte240,    // [7] SMPTE 240 M
    Ycgco,       // [8] YCgCo
    Bt2020Ncl,   // [9] BT.2020 non-constant luminance, BT.2100 YCbCr
    Bt2020Cl,    // [10] BT.2020 constant luminance
    Smpte2085,   // [11] SMPTE ST 2085 YDzDx
    ChromaNcl,   // [12] Chromaticity-derived non-constant luminance
    ChromaCl,    // [13] Chromaticity-derived constant luminance
    Ictcp,       // [14] BT.2100 ICtCp
}

impl Default for MatrixCoefficients {
    fn default() -> Self {
        MatrixCoefficients::Unspecified
    }
}

impl MatrixCoefficients {
    fn as_str(&self) -> &str {
        match self {
            MatrixCoefficients::Identity => "0",
            MatrixCoefficients::Bt709 => "1",
            MatrixCoefficients::Unspecified => "2",
            MatrixCoefficients::Fcc => "4",
            MatrixCoefficients::Bt470bg => "5",
            MatrixCoefficients::Bt601 => "6",
            MatrixCoefficients::Smpte240 => "7",
            MatrixCoefficients::Ycgco => "8",
            MatrixCoefficients::Bt2020Ncl => "9",
            MatrixCoefficients::Bt2020Cl => "10",
            MatrixCoefficients::Smpte2085 => "11",
            MatrixCoefficients::ChromaNcl => "12",
            MatrixCoefficients::ChromaCl => "13",
            MatrixCoefficients::Ictcp => "14",
        }
    }
}

impl AV1Studio {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut style = (*cc.egui_ctx.style()).clone();
        style.text_styles.get_mut(&TextStyle::Body).unwrap().size = 18.0;
        style.text_styles.get_mut(&TextStyle::Heading).unwrap().size = 24.0;

        cc.egui_ctx.set_style(style);

        Self::default()
    }
}

impl eframe::App for AV1Studio {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.max_label_width.is_none() {
            ctx.request_repaint();
            self.max_label_width = Some(0.0);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            set_theme(ctx, MOCHA);

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("AV1Studio");

                ui.separator();

                ui.label(RichText::new("File Options").weak());

                let mut max_width = self.max_label_width.unwrap_or(0.0);

                ui.horizontal(|ui| {
                    let label_text = "Av1an-verbosity Path";
                    let label_width = ui.label(label_text).rect.max.x - ui.min_rect().min.x;
                    max_width = max_width.max(label_width);
                    if label_width < max_width {
                        ui.allocate_space(egui::vec2(max_width - label_width, 1.0));
                    }
                    ui.label(":");
                    ui.text_edit_singleline(&mut self.av1an_verbosity_path);
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
                    let label_text = "*Input File";
                    let label_width = ui.label(label_text).rect.max.x - ui.min_rect().min.x;
                    max_width = max_width.max(label_width);
                    if label_width < max_width {
                        ui.allocate_space(egui::vec2(max_width - label_width, 1.0));
                    }
                    ui.label(":");
                    ui.text_edit_singleline(&mut self.input_file);
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
                    ui.label(":");
                    ui.text_edit_singleline(&mut self.output_file);
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
                    ui.label(":");
                    ui.text_edit_singleline(&mut self.scenes_file);
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
                    ui.label(":");
                    ui.text_edit_singleline(&mut self.zones_file);
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
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui | {
                        ui.label("Full path to a file specifying zones within the video with differing encoder settings. (Check out");
                        ui.hyperlink_to(RichText::new("Trix's Auto Boost Script").color(egui::Color32::from_rgb(4, 165, 229)), "https://github.com/trixoniisama/auto-boost-algorithm");
                            ui.label(".)");
                        });
                    });
                });

                ui.add_space(ui.spacing().item_spacing.y * 2.0);

                ui.label(RichText::new("Source Settings").weak());

                ui.horizontal(|ui| {
                    let label_text = "*Source Library";
                    let label_width = ui.label(label_text).rect.max.x - ui.min_rect().min.x;
                    max_width = max_width.max(label_width);
                    if label_width < max_width {
                        ui.allocate_space(egui::vec2(max_width - label_width, 1.0));
                    }
                    ui.label(":");
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
                    ui.label(":");
                    ui.text_edit_singleline(&mut self.file_concatenation);
                    ui.label(RichText::new("ℹ").weak()).on_hover_ui(|ui| {
                        ui.style_mut().interaction.selectable_labels = true;
                        ui.label("Method to use for concatenating encoded chunks and audio into output file. If you don't know what you're doing, just go with the default option.");
                    });
                });

                ui.add_space(ui.spacing().item_spacing.y * 2.0);

                ui.label(RichText::new("Video Settings").weak());

                ui.horizontal(|ui| {
                    let label_text = "*(Output) Resolution";
                    let label_width = ui.label(label_text).rect.max.x - ui.min_rect().min.x;
                    max_width = max_width.max(label_width);
                    if label_width < max_width {
                        ui.allocate_space(egui::vec2(max_width - label_width, 1.0));
                    }
                    ui.label(":");
                    ui.text_edit_singleline(&mut self.width);
                    ui.label("×");
                    ui.text_edit_singleline(&mut self.height);
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
                    ui.label(":");
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
                        ui.allocate_space(egui::vec2(max_width -label_width, 1.0));
                    }
                    ui.label(":");
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
                                "(2) Unspecified, Default"
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
                    ui.label(":");
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

                ui.add_space(ui.spacing().item_spacing.y * 2.0);

                ui.label(RichText::new("Encoding Parameters").weak());

                ui.horizontal(|ui| {
                    let label_text = "*Preset";
                    let label_width = ui.label(label_text).rect.max.x - ui.min_rect().min.x;
                    max_width = max_width.max(label_width);
                    if label_width < max_width {
                        ui.allocate_space(egui::vec2(max_width - label_width, 1.0));
                    }
                    ui.label(":");
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
                    ui.label(":");
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
                    ui.label(":");
                    ui.text_edit_singleline(&mut self.synthetic_grain);
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
                    }
                    ui.label(":");
                    ui.text_edit_singleline(&mut self.custom_encode_params);
                    ui.label(RichText::new("ℹ").weak()).on_hover_ui(|ui| {
                        ui.style_mut().interaction.selectable_labels = true;
                        ui.label("Provides SVT-AV1-PSY custom encoder parameters on top of the already included parameters.");
                    });
                });

                ui.add_space(ui.spacing().item_spacing.y * 2.0);

                ui.label(RichText::new("Performance Settings").weak());

                ui.horizontal(|ui| {
                    let label_text = "*Thread Affinity";
                    let label_width = ui.label(label_text).rect.max.x - ui.min_rect().min.x;
                    max_width = max_width.max(label_width);
                    if label_width < max_width {
                        ui.allocate_space(egui::vec2(max_width - label_width, 1.0));
                    }
                    ui.label(":");
                    ui.text_edit_singleline(&mut self.thread_affinity);
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
                    ui.label(":");
                    ui.text_edit_singleline(&mut self.workers);
                    ui.label(RichText::new("ℹ").weak()).on_hover_ui(|ui| {
                        ui.style_mut().interaction.selectable_labels = true;
                        ui.label("Number of workers to spawn. It's generally recommended, if you have enough RAM, to set this to the total amount of CPU cores you have for better encoding speeds. Leaving this at the default value will allow Av1an to figure out the amount of workers to spawn automatically.");
                    });
                });

                // Update the stored max width for the next frame
                self.max_label_width = Some(max_width);

                ui.add_space(ui.spacing().item_spacing.y * 2.0);

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

                ctx.request_repaint();
            });
        });
    }
}

fn parse_av1an_output(
    output: &str,
    encoded_frames: &mut Option<u32>,
    total_frames: &mut Option<u32>,
    fps: &mut Option<f64>,
    eta_time: &mut Option<String>,
) {
    println!("parse_av1an_output called with: {}", output);
    let re = Regex::new(r"(\d+)\s+(\d+)").unwrap();

    for line in output.lines() {
        if let Some(caps) = re.captures(line) {
            *encoded_frames = caps.get(1).and_then(|m| m.as_str().parse().ok());
            *total_frames = caps.get(2).and_then(|m| m.as_str().parse().ok());
            *fps = caps.get(3).and_then(|m| m.as_str().parse().ok());
            *eta_time = caps.get(4).map(|m| m.as_str().to_string());
        }
    }
}

fn generate_command(state: &AV1Studio) -> Command {
    let mut cmd = if state.av1an_verbosity_path.is_empty() {
        Command::new("av1an-verbosity")
    } else {
        Command::new(&state.av1an_verbosity_path)
    };

    // Build command arguments
    if !state.input_file.is_empty() {
        cmd.arg("-i").arg(&state.input_file);
    }
    if !state.output_file.is_empty() {
        cmd.arg("-o").arg(&state.output_file);
    }
    if !state.scenes_file.is_empty() {
        cmd.arg("--scenes").arg(&state.scenes_file);
    }
    if !state.zones_file.is_empty() {
        cmd.arg("--zones").arg(&state.zones_file);
    }
    cmd.arg("--verbose-frame-info")
        .arg("--split-method")
        .arg("av-scenechange");

    cmd.arg("-c").arg(if !state.file_concatenation.is_empty() {
        &state.file_concatenation
    } else {
        "mkvmerge"
    });

    cmd.arg("-m")
        .arg(state.source_library.as_str().to_lowercase());

    if !state.width.is_empty() && !state.height.is_empty() {
        let scale = format!(
            "scale={}:{}:flags=bicubic:param0=0:param1=1/2",
            state.width, state.height
        );
        cmd.arg("-f").arg(format!("-vf {}", scale));
    }

    cmd.arg("--pix-format")
        .arg(state.output_pixel_format.as_str())
        .arg("-e")
        .arg("svt-av1");

    if !state.custom_encode_params.is_empty() {
        cmd.arg("-v").arg(&state.custom_encode_params);
    } else {
        let params = format!(
            "--tune 2 --keyint 1 --lp 2 --irefresh-type 2 --crf {} --preset {} --film-grain {}",
            state.crf, state.preset, state.synthetic_grain
        );
        cmd.arg("--force").arg("-v").arg(params);
    }

    cmd.arg("--set-thread-affinity")
        .arg(&state.thread_affinity)
        .arg("-w")
        .arg(&state.workers);

    cmd
}

fn main() -> Result<(), eframe::Error> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "AV1Studio",
        native_options,
        Box::new(|cc| Ok(Box::new(AV1Studio::new(cc)))),
    )
}
