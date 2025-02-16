use iced::{
    executor, Application, Command, Element, Settings, Subscription, Theme,
};
use iced::widget::{button, column, pick_list, row, text, text_input, slider, scrollable};
use iced::theme::Button;
use iced::Renderer;
use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use tokio::sync::mpsc;
use tokio::io::AsyncBufReadExt;
use std::process::{Stdio};
use tokio::process::Command as AsyncCommand;
use regex::Regex;
use std::future::Future;

#[derive(Debug, Clone)]
struct AV1Studio {
    input_path: PathBuf,
    output_path: PathBuf,
    scenes_path: Option<PathBuf>,
    zones_path: Option<PathBuf>,
    source_module: SourceModule,
    resolution: (Option<u32>, Option<u32>),
    pixel_format: PixelFormat,
    audio_encoder: AudioEncoder,
    audio_bitrate: String,
    crf: u8,
    preset: u8,
    workers: u16,
    thread_affinity: u16,
    film_grain: u8,
    encoding_state: EncodingState,
    custom_params: String,
    current_theme: Theme,
}

#[derive(Debug, Clone)]
enum Message {
    ChangeTheme(Theme),
    SelectInput,
    SelectOutput,
    SelectScenes,
    SelectZones,
    InputFileSelected(Option<PathBuf>),
    OutputFileSelected(Option<PathBuf>),
    ScenesFileSelected(Option<PathBuf>),
    ZonesFileSelected(Option<PathBuf>),
    SourceModuleChanged(SourceModule),
    ResolutionWidthChanged(String),
    ResolutionHeightChanged(String),
    PixelFormatChanged(PixelFormat),
    AudioBitrateChanged(String),
    CrfChanged(String),
    PresetChanged(String),
    WorkersChanged(String),
    ThreadAffinityChanged(String),
    FilmGrainChanged(String),
    StartEncoding,
    EncodingProgress(ProgressUpdate),
    EncodingFinished(Result<(), String>),
    CancelEncoding,
    AudioEncoderChanged(AudioEncoder),
    CustomParamsChanged(String),
    SelectPreset,
    SavePreset,
    PresetFileSelected(Option<PathBuf>),
    SavePresetFileSelected(Option<PathBuf>),
}

#[derive(Debug, Clone, Default)]
struct ProgressUpdate {
    status: String,
    percentage: f32,
    current_frame: u64,
    total_frames: u64,
    fps: f32,
    eta: String,
    elapsed_time: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct Preset {
    source_module: SourceModule,
    resolution: (Option<u32>, Option<u32>),
    pixel_format: PixelFormat,
    audio_encoder: AudioEncoder,
    audio_bitrate: String,
    crf: u8,
    preset: u8,
    workers: u16,
    thread_affinity: u16,
    film_grain: u8,
    custom_params: String,
}

#[derive(Debug, Clone)]
enum EncodingState {
    Idle,
    Encoding {
        status: String,
        progress: f32,
        current_frame: u64,
        total_frames: u64,
        fps: f32,
        eta: String,
        cancel_sender: mpsc::Sender<()>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
enum SourceModule {
    #[default]
    FFMS2,
    BestSource,
    LSMash,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
enum PixelFormat {
    Yuv420p,
    #[default]
    Yuv420p10le,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
enum AudioEncoder {
    #[default]
    OPUS,
    AAC,
    Vorbis,
}

impl EncodingState {
    fn get_cancel_sender(&self) -> Option<mpsc::Sender<()>> {
        if let EncodingState::Encoding { cancel_sender, .. } = self {
            Some(cancel_sender.clone())
        } else {
            None
        }
    }
}

impl SourceModule {
    const ALL: [SourceModule; 3] = [
        SourceModule::FFMS2,
        SourceModule::BestSource,
        SourceModule::LSMash,
    ];
}

impl std::fmt::Display for SourceModule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SourceModule::FFMS2 => write!(f, "FFMS2"),
            SourceModule::BestSource => write!(f, "BestSource"),
            SourceModule::LSMash => write!(f, "LSMASH"),
        }
    }
}

impl PixelFormat {
    const ALL: [PixelFormat; 2] = [
        PixelFormat::Yuv420p,
        PixelFormat::Yuv420p10le,
    ];
}

impl std::fmt::Display for PixelFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PixelFormat::Yuv420p => write!(f, "yuv420p"),
            PixelFormat::Yuv420p10le => write!(f, "yuv420p10le"),
        }
    }
}

impl AudioEncoder {
    const ALL: [AudioEncoder; 3] = [
        AudioEncoder::OPUS,
        AudioEncoder::AAC,
        AudioEncoder::Vorbis,
    ];
}

impl std::fmt::Display for AudioEncoder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioEncoder::OPUS => write!(f, "OPUS"),
            AudioEncoder::AAC => write!(f, "AAC"),
            AudioEncoder::Vorbis => write!(f, "Vorbis"),
        }
    }
}

impl AV1Studio {
    fn generate_av1an_command(&self) -> String {
        let input = self.input_path.to_string_lossy();
        let output = self.output_path.to_string_lossy();

        let _resolution = match self.resolution {
            (Some(width), Some(height)) => format!("scale={}:{}", width, height),
            _ => String::new(),
        };

        let video_params = format!(
            "--tune 2 --keyint 1 --lp 2 --irefresh-type 2 --crf {} --preset {} --film-grain {}",
            self.crf, self.preset, self.film_grain,
        );

        let audio_params = match self.audio_encoder {
            AudioEncoder::OPUS => format!("-c:a libopus -b:a {}k -ac 2", self.audio_bitrate),
            AudioEncoder::AAC => format!("-c:a aac -b:a {}k -ac 2", self.audio_bitrate),
            AudioEncoder::Vorbis => format!("-c:a libvorbis -b:a {}k -ac 2", self.audio_bitrate),
        };

        let zones_param = self.zones_path.as_ref().map(|path| 
            format!("--zones \"{}\" ", path.to_string_lossy())
        ).unwrap_or_default();

        let scenes_param = self.scenes_path.as_ref().map(|path| 
            format!("--scenes \"{}\" ", path.to_string_lossy())
        ).unwrap_or_default();

        let resolution = match self.resolution {
            (Some(w), Some(h)) => format!("-f \"-vf scale={}:{}:flags=bicubic:param0=0:param1=1/2\"", w, h),
            _ => String::new(),
        };

        let thread_affinity_param = if self.thread_affinity > 0 {
            format!("--set-thread-affinity {} ", self.thread_affinity)
        } else {
            String::new()
        };

        format!(
            "av1an-verbosity -i \"{}\" {} {} --verbose-frame-info --sc-pix-format=yuv420p --split-method av-scenechange -m {} -c mkvmerge --sc-downscale-height 1080 -e svt-av1 --force -v \"{} {}\" --pix-format {} {} -a \"{}\" {} -w {} -o \"{}\"",
            input,
            zones_param,
            scenes_param,
            self.source_module.to_string().to_lowercase(),
            video_params,
            self.custom_params,
            self.pixel_format,
            resolution,
            audio_params,
            thread_affinity_param,
            self.workers,
            output
        )
    }

    fn to_preset(&self) -> Preset {
        Preset {
            source_module: self.source_module,
            resolution: self.resolution,
            pixel_format: self.pixel_format,
            audio_encoder: self.audio_encoder,
            audio_bitrate: self.audio_bitrate.clone(),
            crf: self.crf,
            preset: self.preset,
            workers: self.workers,
            thread_affinity: self.thread_affinity,
            film_grain: self.film_grain,
            custom_params: self.custom_params.clone(),
        }
    }

    fn apply_preset(&mut self, preset: Preset) {
        self.source_module = preset.source_module;
        self.resolution = preset.resolution;
        self.pixel_format = preset.pixel_format;
        self.audio_encoder = preset.audio_encoder;
        self.audio_bitrate = preset.audio_bitrate;
        self.crf = preset.crf;
        self.preset = preset.preset;
        self.workers = preset.workers;
        self.thread_affinity = preset.thread_affinity;
        self.film_grain = preset.film_grain;
        self.custom_params = preset.custom_params;
    }

    async fn run_encoding(command: String, mut cancel_rx: mpsc::Receiver<()>, progress_tx: mpsc::Sender<ProgressUpdate>) -> Result<(), String> {
        println!("Starting encoding with command: {}", command);
    
        let mut child = AsyncCommand::new("sh")
            .arg("-c")
            .arg(command)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| e.to_string())?;
    
        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();
        
        let mut stdout_reader = tokio::io::BufReader::new(stdout);
        let mut stderr_reader = tokio::io::BufReader::new(stderr);
        
        let mut stdout_line = String::new();
        let mut stderr_line = String::new();
    
        tokio::select! {
            status = async {
                loop {
                    tokio::select! {
                        result = stdout_reader.read_line(&mut stdout_line) => {
                            if let Ok(n) = result {
                                if n == 0 { break; }
                                println!("Stdout: {}", stdout_line); 
                                
                                if stdout_line.trim().matches(' ').count() == 1 
                                   && stdout_line.trim().split(' ').all(|s| s.parse::<u64>().is_ok()) {
                                    if let Some(progress) = Self::parse_progress(&stdout_line) {
                                        println!("Sending frame count update: {:?}", progress);
                                        let _ = progress_tx.send(progress).await;
                                    }
                                }
                                stdout_line.clear();
                            }
                        }
                        result = stderr_reader.read_line(&mut stderr_line) => {
                            if let Ok(n) = result {
                                if n == 0 { break; }
                                println!("Stderr: {}", stderr_line); 
                                
                                if stderr_line.contains('▕') && stderr_line.contains('%') {
                                    if let Some(progress) = Self::parse_progress(&stderr_line) {
                                        println!("Sending progress bar update: {:?}", progress);
                                        let _ = progress_tx.send(progress).await;
                                    }
                                }
                                stderr_line.clear();
                            }
                        }
                    }
                }
                child.wait().await
            } => {
                match status {
                    Ok(status) if status.success() => Ok(()),
                    Ok(status) => Err(format!("Process exited with status: {}", status)),
                    Err(e) => Err(e.to_string()),
                }
            }
            _ = cancel_rx.recv() => {
                child.kill().await.map_err(|e| e.to_string())?;
                Err("Encoding canceled".to_string())
            }
        }
    }

    fn parse_progress(line: &str) -> Option<ProgressUpdate> {
        let frame_re = Regex::new(r"^(\d+)\s+(\d+)$").ok()?;
        
        let progress_re = Regex::new(r"[⠀-⣿]?\s*(\d+:\d+:\d+)\s+▕[█▏▎▍▌▋▊▉\s]*▏\s*(\d+)%\s*(\d+)/(\d+)\s*\(([\d\.]+)(?:\s*fps|(?:\s*s/fr)),\s*eta\s*([^\)]+)\)").ok()?;
    
        let line = line.trim();
    
        if let Some(caps) = frame_re.captures(line) {
            let current_frame: u64 = caps.get(1)?.as_str().parse().ok()?;
            let total_frames: u64 = caps.get(2)?.as_str().parse().ok()?;
            let percentage = (current_frame as f32 / total_frames as f32) * 100.0;
            
            println!("Parsed frame count: {}/{} ({}%)", current_frame, total_frames, percentage); 
            
            return Some(ProgressUpdate {
                status: String::new(),
                percentage,
                current_frame,
                total_frames,
                fps: 0.0,
                eta: String::new(),
                elapsed_time: String::new(),
            });
        } else if let Some(caps) = progress_re.captures(line) {
            let fps_str = caps.get(5)?.as_str();
            let fps = if fps_str.contains("s/fr") {
                let spf: f32 = fps_str.parse().ok()?;
                if spf > 0.0 { 1.0 / spf } else { 0.0 }
            } else {
                fps_str.parse().ok()?
            };
            
            println!("Parsed progress line: {}% FPS: {} ETA: {}", 
                caps.get(2)?.as_str(),
                fps,
                caps.get(6)?.as_str()
            ); 
            
            return Some(ProgressUpdate {
                status: String::new(),
                percentage: caps.get(2)?.as_str().parse().ok()?,
                current_frame: caps.get(3)?.as_str().parse().ok()?,
                total_frames: caps.get(4)?.as_str().parse().ok()?,
                fps,
                eta: caps.get(6)?.as_str().to_string(),
                elapsed_time: caps.get(1)?.as_str().to_string(),
            });
        }
    
        None
    }
}

fn handle_progress(mut progress_rx: mpsc::Receiver<ProgressUpdate>) -> impl Future<Output = ProgressUpdate> {
    async move {
        while let Some(progress) = progress_rx.recv().await {
            return progress;
        }
        ProgressUpdate::default() 
    }
}

impl Application for AV1Studio {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let total_cores = num_cpus::get_physical() as u16;

        (
            Self {
                input_path: PathBuf::new(),
                output_path: PathBuf::new(),
                scenes_path: None,
                zones_path: None,
                source_module: SourceModule::FFMS2,
                resolution: (None, None),
                pixel_format: PixelFormat::Yuv420p10le,
                audio_encoder: AudioEncoder::default(),
                audio_bitrate: "128".to_string(),
                crf: 29,
                preset: 4,
                workers: total_cores,
                thread_affinity: 2,
                film_grain: 0,
                encoding_state: EncodingState::Idle,
                custom_params: String::new(),
                current_theme: Theme::Dark,
            },
            Command::none(),
        )
    }

    fn theme(&self) -> Theme {
        self.current_theme.clone()
    }

    fn title(&self) -> String {
        String::from("AV1Studio")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::ChangeTheme(theme) => {
                self.current_theme = theme;
                Command::none()
            }
            Message::SelectInput => Command::perform(
                async {
                    rfd::AsyncFileDialog::new()
                        .add_filter("MKV files", &["mkv"])
                        .pick_file()
                        .await
                        .map(|f| f.path().to_owned())
                },
                Message::InputFileSelected,
            ),
            Message::SelectOutput => Command::perform(
                async {
                    rfd::AsyncFileDialog::new()
                        .add_filter("MKV files", &["mkv"])
                        .save_file()
                        .await
                        .map(|f| f.path().to_owned())
                },
                Message::OutputFileSelected,
            ),
            Message::InputFileSelected(path) => {
                if let Some(path) = path {
                    self.input_path = path;
                }
                Command::none()
            }
            Message::OutputFileSelected(path) => {
                if let Some(path) = path {
                    self.output_path = path;
                }
                Command::none()
            }
            Message::SourceModuleChanged(library) => {
                self.source_module = library;
                Command::none()
            }
            Message::ResolutionWidthChanged(input) => {
                self.resolution.0 = input.parse().ok();
                Command::none()
            }
            Message::ResolutionHeightChanged(input) => {
                self.resolution.1 = input.parse().ok();
                Command::none()
            }
            Message::PixelFormatChanged(module) => {
                self.pixel_format = module;
                Command::none()
            }
            Message::CrfChanged(input) => {
                if let Ok(value) = input.parse::<u8>() {
                    self.crf = value.clamp(0, 63);
                }
                Command::none()
            }
            Message::PresetChanged(input) => {
                if let Ok(value) = input.parse::<u8>() {
                    self.preset = value.clamp(0, 13);
                }
                Command::none()
            }
            Message::FilmGrainChanged(input) => {
                if let Ok(value) = input.parse::<u8>() {
                    self.film_grain = value.clamp(0, 10);
                }
                Command::none()
            }
            Message::AudioEncoderChanged(encoder) => {
                self.audio_encoder = encoder;
                Command::none()
            }
            Message::AudioBitrateChanged(input) => {
                if input.chars().all(|c| c.is_ascii_digit()) {
                    self.audio_bitrate = input;
                }
                Command::none()
            }
            Message::CustomParamsChanged(params) => {
                self.custom_params = params;
                Command::none()
            }
            Message::ThreadAffinityChanged(input) => {
                if let Ok(value) = input.parse::<u16>() {
                    self.thread_affinity = value;
                }
                Command::none()
            }
            Message::WorkersChanged(input) => {
                if let Ok(value) = input.parse::<u16>() {
                    self.workers = value;
                }
                Command::none()
            }
            Message::StartEncoding => {
                let command = self.generate_av1an_command();
                let (cancel_tx, cancel_rx) = mpsc::channel(1);
                let (progress_tx, mut progress_rx) = mpsc::channel(100);
            
                self.encoding_state = EncodingState::Encoding {
                    status: String::new(),
                    progress: 0.0,
                    current_frame: 0,
                    total_frames: 0,
                    fps: 0.0,
                    eta: String::new(),
                    cancel_sender: cancel_tx,
                };
            
                Command::batch([
                    Command::perform(
                        Self::run_encoding(command, cancel_rx, progress_tx),
                        Message::EncodingFinished,
                    ),
                    Command::perform(
                        async move {
                            while let Some(progress) = progress_rx.recv().await {
                                println!("StartEncoding: Received progress: {:?}", progress);
                                return progress;
                            }
                            ProgressUpdate::default()
                        },
                        Message::EncodingProgress,
                    ),
                ])
            }
            Message::EncodingProgress(progress) => {
                println!("UPDATE: Received progress: {:?}", progress);
                if let EncodingState::Encoding { cancel_sender, .. } = &self.encoding_state {
                    println!("UPDATE: Updating encoding state");
                    self.encoding_state = EncodingState::Encoding {
                        status: progress.status,
                        progress: progress.percentage / 100.0,
                        current_frame: progress.current_frame,
                        total_frames: progress.total_frames,
                        fps: progress.fps,
                        eta: progress.eta,
                        cancel_sender: cancel_sender.clone(),
                    };
                    println!("UPDATE: New state: {:?}", self.encoding_state);
                }
                Command::none()
            }
            Message::CancelEncoding => {
                if let EncodingState::Encoding { cancel_sender, .. } = &self.encoding_state {
                    let _ = cancel_sender.try_send(());
                }
                self.encoding_state = EncodingState::Idle;
                Command::none()
            }
            Message::EncodingFinished(result) => {
                self.encoding_state = EncodingState::Idle;
                match result {
                    Ok(_) => println!("Encoding completed successfully"),
                    Err(e) => eprintln!("Encoding failed: {}", e),
                }
                Command::none()
            }
            Message::SelectScenes => Command::perform(
                async {
                    rfd::AsyncFileDialog::new()
                        .add_filter("JSON files", &["json"])
                        .pick_file()
                        .await
                        .map(|f| f.path().to_owned())
                },
                Message::ScenesFileSelected,
            ),
            Message::SelectZones => Command::perform(
                async {
                    rfd::AsyncFileDialog::new()
                        .add_filter("Text files", &["txt"])
                        .pick_file()
                        .await
                        .map(|f| f.path().to_owned())
                },
                Message::ZonesFileSelected,
            ),
            Message::ScenesFileSelected(path) => {
                self.scenes_path = path;
                Command::none()
            }
            Message::ZonesFileSelected(path) => {
                self.zones_path = path;
                Command::none()
            }
            Message::SelectPreset => Command::perform(
                async {
                    rfd::AsyncFileDialog::new()
                        .add_filter("JSON files", &["json"])
                        .pick_file()
                        .await
                        .map(|f| f.path().to_owned())
                },
                Message::PresetFileSelected,
            ),
            Message::SavePreset => Command::perform(
                async {
                    rfd::AsyncFileDialog::new()
                        .add_filter("JSON files", &["json"])
                        .save_file()
                        .await
                        .map(|f| f.path().to_owned())
                },
                Message::SavePresetFileSelected,
            ),
            Message::PresetFileSelected(path) => {
                if let Some(path) = path {
                    if let Ok(file) = std::fs::read_to_string(&path) {
                        if let Ok(preset) = serde_json::from_str::<Preset>(&file) {
                            self.apply_preset(preset);
                        }
                    }
                }
                Command::none()
            }
            Message::SavePresetFileSelected(path) => {
                if let Some(path) = path {
                    let preset = self.to_preset();
                    if let Ok(json) = serde_json::to_string_pretty(&preset) {
                        let _ = std::fs::write(path, json);
                    }
                }
                Command::none()
            }
            _ => Command::none(),
        }
    }

    fn view(&self) -> Element<Self::Message> {
        let input_text = if self.input_path.as_os_str().is_empty() {
            "No input file selected".to_string()
        } else {
            self.input_path.to_string_lossy().to_string()
        };

        let output_text = if self.output_path.as_os_str().is_empty() {
            "No output file selected".to_string()
        } else {
            self.output_path.to_string_lossy().to_string()
        };

        let source_selector = row![
            text("Source Library:").width(100),
            pick_list(SourceModule::ALL, Some(self.source_module), Message::SourceModuleChanged).width(200)
        ]
        .spacing(10)
        .padding(5);

        let resolution_controls = row![
            text("(Output) Resolution:").width(100),
            text_input(
                "Width",
                &self.resolution.0.map(|w| w.to_string()).unwrap_or_default()
            )
            .on_input(Message::ResolutionWidthChanged)
            .width(100),
            text("×"),
            text_input(
                "Height",
                &self.resolution.1.map(|h| h.to_string()).unwrap_or_default()
            )
            .on_input(Message::ResolutionHeightChanged)
            .width(100),
        ]
        .spacing(10)
        .padding(5);

        let pixel_format_selector = row![
            text("(Output) Pixel Format:").width(100),
            pick_list(PixelFormat::ALL, Some(self.pixel_format), Message::PixelFormatChanged).width(200)
        ]
        .spacing(10)
        .padding(5);

        let crf_control = row![
            text("CRF:").width(100),
            slider(0..=63, self.crf, |value| Message::CrfChanged(value.to_string())).width(500),
            text(format!("{}", self.crf)).width(30),
        ]
        .spacing(10)
        .padding(5);

        let encoding_controls: Element<Message> = match &self.encoding_state {
            EncodingState::Encoding { progress, fps, eta, current_frame, total_frames, .. } => {
                println!("VIEW: Rendering encoding state: progress={}, frames={}/{}", 
                    progress, current_frame, total_frames);
                
                column![
                    row![
                        button::<Message, Theme, Renderer>("Cancel Encoding")  
                            .on_press(Message::CancelEncoding)
                            .padding(10)
                            .style(Button::Destructive),
                        slider(0.0..=1.0, *progress, |_| Message::CancelEncoding)
                            .width(500),
                        text(format!("{:.1}%", progress * 100.0)).width(50)
                    ]
                    .spacing(10)
                    .padding(5),
                    row![
                        text(format!("Frames: {}/{}", current_frame, total_frames)).width(200),
                        text(if *fps > 0.0 { 
                            format!("FPS: {:.1}", fps) 
                        } else { 
                            "FPS: -".to_string() 
                        }).width(100),
                        text(if !eta.is_empty() { 
                            format!("ETA: {}", eta) 
                        } else { 
                            "ETA: -".to_string() 
                        }).width(100),
                    ]
                    .spacing(10)
                    .padding(5)
                ].into()
            }
            EncodingState::Idle => {
                println!("VIEW: Rendering idle state");
                row![
                    button::<Message, Theme, Renderer>("Start Encoding")  
                        .on_press(Message::StartEncoding)
                        .padding(10)
                        .style(Button::Primary)
                ].into()
            }
        };

        let preset_control = row![
            text("Preset:").width(100),
            slider(0..=13, self.preset, |value| Message::PresetChanged(value.to_string())).width(500),
            text(format!("{}", self.preset)).width(30),
        ]
        .spacing(10)
        .padding(5);

        let film_grain_control = row![
            text("(Output) Film Grain:").width(100),
            slider(0..=10, self.film_grain, |value| Message::FilmGrainChanged(value.to_string())).width(500),
            text(format!("{}", self.film_grain)).width(30),
        ]
        .spacing(10)
        .padding(5);

        let audio_controls: Element<_> = row![
            text("Audio Encoder Settings:").width(100),
            pick_list(AudioEncoder::ALL, Some(self.audio_encoder), Message::AudioEncoderChanged).width(100),
            text_input("Bitrate", &self.audio_bitrate)
            .on_input(Message::AudioBitrateChanged)
            .width(100),
            text("(K)").width(0)
        ]
        .spacing(10)
        .padding(5)
        .into();

        let custom_params_control = row![
            text("Custom Encoder Params:").width(100),
            text_input("Enter custom encoder params", &self.custom_params)
            .on_input(Message::CustomParamsChanged)
            .width(500),
        ]
        .spacing(10)
        .padding(5);

        let av1an_options = column![
            text("Av1an Options:").size(16),
            row![
                text("Thread Affinity:").width(100),
                text_input("2", &self.thread_affinity.to_string())
                .on_input(Message::ThreadAffinityChanged)
                .width(100),
            ]
            .spacing(10)
            .padding(5),
            row![
                text("Workers:").width(100),
                text_input("0", &self.workers.to_string())
                .on_input(Message::WorkersChanged)
                .width(100),
            ]
            .spacing(10)
            .padding(5),
        ]
        .spacing(10)
        .padding(5);

        let scenes_text = self.scenes_path.as_ref().map_or(
            "No scenes file selected".to_string(),
            |path| path.to_string_lossy().to_string()
        );

        let zones_text = self.zones_path.as_ref().map_or(
            "No zones file selected".to_string(),
            |path| path.to_string_lossy().to_string()
        );

        let preset_controls = row![
            button("Load Preset")
            .on_press(Message::SelectPreset),
            button("Save Preset")
            .on_press(Message::SavePreset),
        ]
        .spacing(10)
        .padding(5);

        let theme_switcher = row![
            button("Light Theme")
            .on_press(Message::ChangeTheme(Theme::Light)),
            button("Dark Theme")
            .on_press(Message::ChangeTheme(Theme::Dark)),
        ]
        .spacing(10)
        .padding(5);

        scrollable(
            column![
                theme_switcher,

                text("AV1Studio").size(24),
                text("A GUI for AV1 encoding via Av1an and SVT-AV1-PSY written in Rust using iced.").size(16),
                row![
                    text("Input File:").width(100),
                    text(input_text).width(400),
                    button("Select").on_press(Message::SelectInput),
                ]
                .spacing(10)
                .padding(5),
            
                row![
                    text("Output File:").width(100),
                    text(output_text).width(400),
                    button("Select").on_press(Message::SelectOutput),
                ]
                .spacing(10)
                .padding(5),

                row![
                    text("Scenes File:").width(100),
                    text(scenes_text).width(400),
                    button("Select").on_press(Message::SelectScenes),
                ]
                .spacing(10)
                .padding(5),

                row![
                    text("Zones File:").width(100),
                    text(zones_text).width(400),
                    button("Select").on_press(Message::SelectZones),
                ]
                .spacing(10)
                .padding(5),

                source_selector,

                resolution_controls,

                pixel_format_selector,

                preset_control,

                crf_control,

                film_grain_control,

                custom_params_control,

                audio_controls,

                av1an_options,

                encoding_controls,

                preset_controls,
            ]
            .padding(20)
            .spacing(20)
        )
        .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        match &self.encoding_state {
            EncodingState::Encoding { .. } => {
                println!("SUBSCRIPTION: Active");
                iced::subscription::unfold(
                    "encoding_progress",
                    None::<ProgressUpdate>,
                    |state| async {
                        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                        match state {
                            Some(progress) => (
                                Message::EncodingProgress(progress),
                                None
                            ),
                            None => (
                                Message::EncodingProgress(ProgressUpdate::default()),
                                None
                            )
                        }
                    }
                )
            }
            EncodingState::Idle => {
                println!("SUBSCRIPTION: Idle");
                Subscription::none()
            }
        }
    }
}

#[tokio::main]
async fn main() -> iced::Result {
    AV1Studio::run(Settings::default())
}
