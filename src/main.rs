use iced::{
    executor, Application, Command, Element, Settings, Subscription, Theme,
};
use iced::widget::{button, column, pick_list, row, text, text_input, slider, scrollable};
use std::path::PathBuf;
use tokio::sync::mpsc;
use num_cpus::{{get, get_physical}};

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
}

#[derive(Debug, Clone)]
enum Message {
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
}

#[derive(Debug, Clone, Default)]
enum EncodingState {
    #[default]
    Idle,
    Encoding {
        progress: f32,
        current_frame: u64,
        total_frames: u64,
        cancel_sender: mpsc::Sender<()>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum SourceModule {
    #[default]
    FFMS2,
    BestSource,
    LSMash,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum PixelFormat {
    Yuv420p,
    #[default]
    Yuv420p10le,
}

#[derive(Debug, Clone)]
enum ProgressUpdate {
    Progress { current: u64, total: u64 },
    Finished,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum AudioEncoder {
    #[default]
    OPUS,
    AAC,
    Vorbis,
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

        let resolution = match self.resolution {
            (Some(w), Some(h)) => format!("-f \"-vf scale={}:{}:flags=bicubic:param0=0:param1=1/2\"", w, h),
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
            "av1an -i \"{}\" {} {} --verbose --sc-pix-format=yuv420p --split-method --av-scenechange -m {} -c mkvmerge --sc-downscale-height 1080 -e svt-av1 --force -v \"{} {}\" --pix-format {} {} -a \"{}\" {} -w {} -o \"{}\"",
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
}

impl Application for AV1Studio {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let total_cores = num_cpus::get_physical() as u16;
        let total_threads = num_cpus::get() as u16;
        let thread_affinity = (total_threads / total_cores);

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
                thread_affinity,
                film_grain: 0,
                encoding_state: EncodingState::Idle,
                custom_params: String::new(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("AV1Studio")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
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
                if self.input_path.exists() {
                    let command = self.generate_av1an_command();
                    println!("Generated command: \n{}", command);
                    Command::none()
                } else {
                    Command::none()
                }
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

        let encoding_controls = match &self.encoding_state {
            EncodingState::Idle => row![
                button("Start Encoding")
                .on_press(Message::StartEncoding)
                .padding(10)
            ],
            EncodingState::Encoding { progress, .. } => row![
                button("Cancel Encoding")
                .on_press(Message::CancelEncoding)
                .padding(10),
                slider(0.0..=1.0, *progress, |_| Message::EncodingProgress(ProgressUpdate::Progress { current: 0, total: 0 }))
                .width(500),
                text(format!("{:.1}%", progress * 100.0)).width(50)
            ]
            .spacing(10)
            .padding(5),
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

        scrollable(
            column![
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
            ]
            .padding(20)
            .spacing(20)
        )
        .into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::none()
    }
}

fn main() -> iced::Result {
    AV1Studio::run(Settings::default())
}
