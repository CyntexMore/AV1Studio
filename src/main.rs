mod app;
mod encoding;
mod models;

fn main() -> Result<(), eframe::Error> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "AV1Studio",
        native_options,
        Box::new(|cc| Ok(Box::new(app::AV1Studio::new(cc)))),
    )
}
