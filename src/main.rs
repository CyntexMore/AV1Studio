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
            file_concatenation: String::new(),
            preset: 4.0,
            crf: 29.0,
            synthetic_grain: 0.to_string(),
            custom_encode_params: String::new(),
            thread_affinity: 2.to_string(),
            workers: 6.to_string(),
            encoded_frames: None,
            total_frames: None,
            fps: None,
            eta_time: None,
            encoding_in_progress: false,
            receiver: None,
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
        egui::CentralPanel::default().show(ctx, |ui| {
            set_theme(ctx, MOCHA);

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("AV1Studio");

                ui.separator();

                ui.label(RichText::new("File Options").weak());

                ui.horizontal(|ui| {
                    ui.label("Av1an-verbosity Path:");
                    ui.text_edit_singleline(&mut self.av1an_verbosity_path);
                });

                ui.horizontal(|ui| {
                    ui.label("*Input File:");
                    ui.text_edit_singleline(&mut self.input_file);
                    if ui.button("Browse").clicked() {
                        if let Some(path) = FileDialog::new()
                            .add_filter("Video Files", &[".mkv"])
                            .pick_file()
                        {
                            self.input_file = path.display().to_string();
                        }
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("*Output File:");
                    ui.text_edit_singleline(&mut self.output_file);
                    if ui.button("Browse").clicked() {
                        if let Some(path) = FileDialog::new()
                            .add_filter("Video Files", &["mkv"])
                            .pick_file()
                        {
                            self.output_file = path.display().to_string();
                        }
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Scenes File:");
                    ui.text_edit_singleline(&mut self.scenes_file);
                    if ui.button("Browse").clicked() {
                        if let Some(path) = FileDialog::new()
                            .add_filter("JSON Files", &["json"])
                            .pick_file()
                        {
                            self.scenes_file = path.display().to_string();
                        }
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Zones File:");
                    ui.text_edit_singleline(&mut self.zones_file);
                    if ui.button("Browse").clicked() {
                        if let Some(path) = FileDialog::new()
                            .add_filter("TXT Files", &["txt"])
                            .pick_file()
                        {
                            self.zones_file = path.display().to_string();
                        }
                    }
                });

                ui.add_space(ui.spacing().item_spacing.y * 2.0);

                ui.label(RichText::new("Source Settings").weak());

                ui.horizontal(|ui| {
                    ui.label("*Source Library:");
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
                });

                ui.horizontal(|ui| {
                    ui.label("File Concatenation:");
                    ui.text_edit_singleline(&mut self.file_concatenation);
                });

                ui.add_space(ui.spacing().item_spacing.y * 2.0);

                ui.label(RichText::new("Video Settings").weak());

                ui.horizontal(|ui| {
                    ui.label("*(Output) Resolution:");
                    ui.text_edit_singleline(&mut self.width);
                    ui.label("×");
                    ui.text_edit_singleline(&mut self.height);
                });

                ui.horizontal(|ui| {
                    ui.label("*(Output) Pixel Format:");
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
                });

                ui.add_space(ui.spacing().item_spacing.y * 2.0);

                ui.label(RichText::new("Encoding Parameters").weak());

                ui.horizontal(|ui| {
                    ui.label("*Preset:");
                    ui.add(
                        Slider::new(&mut self.preset, 0.0..=13.0)
                            .step_by(1.0)
                            .custom_formatter(|n, _| format!("{}", n as i32)),
                    );
                });

                ui.horizontal(|ui| {
                    ui.label("*CRF:");
                    ui.add(Slider::new(&mut self.crf, 0.0..=63.0).step_by(1.0));
                });

                ui.horizontal(|ui| {
                    ui.label("*Synthetic Grain:");
                    ui.text_edit_singleline(&mut self.synthetic_grain);
                });

                ui.horizontal(|ui| {
                    ui.label("Custom Encoder Parameters:");
                    ui.text_edit_singleline(&mut self.custom_encode_params);
                });

                ui.add_space(ui.spacing().item_spacing.y * 2.0);

                ui.label(RichText::new("Performance Settings").weak());

                ui.horizontal(|ui| {
                    ui.label("*Thread Affinity:");
                    ui.text_edit_singleline(&mut self.thread_affinity);
                });

                ui.horizontal(|ui| {
                    ui.label("*Workers:");
                    ui.text_edit_singleline(&mut self.workers);
                });

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
