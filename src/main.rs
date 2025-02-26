use egui::widgets::{Button, Slider};
use egui::{Color32, ComboBox, Style, TextStyle, Ui, Visuals};

#[derive(Default)]
struct AV1Studio {
    // TODO: Add file dialogs with rfd
    input_file: String,
    output_file: String,
    scenes_file: String,
    zones_file: String,

    source_library: SourceLibrary,

    width: String,
    height: String,

    output_pixel_format: PixelFormat,

    preset: f32,
    crf: f32,
    synthetic_grain: String, // Synthetic grain is a String to allow editing
    custom_encode_params: String,
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum SourceLibrary {
    BestSource,
    FFMS2,
    LSMASH,
}

impl Default for SourceLibrary {
    fn default() -> Self {
        SourceLibrary::BestSource
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
        PixelFormat::Yuv420p
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
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("AV1Studio");
            ui.separator();

            ui.label("File Options");
            ui.horizontal(|ui| {
                ui.label("Input File:");
                ui.text_edit_singleline(&mut self.input_file);
            });

            ui.horizontal(|ui| {
                ui.label("Output File:");
                ui.text_edit_singleline(&mut self.output_file);
            });

            ui.horizontal(|ui| {
                ui.label("Scenes File:");
                ui.text_edit_singleline(&mut self.scenes_file);
            });

            ui.horizontal(|ui| {
                ui.label("Zones File:");
                ui.text_edit_singleline(&mut self.zones_file);
            });

            ui.separator();

            ui.label("Source Library:");
            ComboBox::from_label("")
                .selected_text(self.source_library.as_str())
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.source_library,
                        SourceLibrary::BestSource,
                        "BestSource",
                    );
                    ui.selectable_value(&mut self.source_library, SourceLibrary::FFMS2, "FFMS2");
                    ui.selectable_value(&mut self.source_library, SourceLibrary::LSMASH, "L-SMASH");
                });

            ui.separator();

            ui.label("(Output) Resolution:");
            ui.horizontal(|ui| {
                ui.label("Width:");
                ui.text_edit_singleline(&mut self.width);
                ui.label("×");
                ui.label("Height:");
                ui.text_edit_singleline(&mut self.height);
            });

            ui.separator();

            ui.label("Preset:");
            ui.add(Slider::new(&mut self.preset, 0.0..=13.0).step_by(1.0));

            ui.label("CRF:");
            ui.add(Slider::new(&mut self.crf, 0.0..=63.0).step_by(1.0));

            ui.separator();

            ui.label("Synthetic Grain:");
            ui.text_edit_singleline(&mut self.synthetic_grain);

            ui.separator();

            ui.label("Custom Encoder Parameters:");
            ui.text_edit_multiline(&mut self.custom_encode_params);

            ui.separator();

            if ui.button("Start Encoding").clicked() {
                println!("Start Encoding button pressed");

                start_encoding(&self);
            }
        });
    }
}

fn start_encoding(state: &AV1Studio) {
    println!("Encoding with parameters:");
    println!("Input File: {}", state.input_file);
    println!("Output FIle: {}", state.output_file);
}

fn main() -> Result<(), eframe::Error> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "AV1Studio",
        native_options,
        Box::new(|cc| Ok(Box::new(AV1Studio::new(cc)))),
    )
}
