use eframe::{egui, epi};

fn main() {
    simple_logger::SimpleLogger::new().init().unwrap();

    let app = SettingsGuiDemoApp::default();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}

#[derive(Default)]
pub struct SettingsGuiDemoApp {}

impl epi::App for SettingsGuiDemoApp {
    fn name(&self) -> &str {
        "rusty-daw-io settings GUI demo"
    }

    /// Called once before the first frame.
    fn setup(
        &mut self,
        _ctx: &egui::CtxRef,
        _frame: &epi::Frame,
        _storage: Option<&dyn epi::Storage>,
    ) {
        dbg!(rainout::available_audio_backends());
        dbg!(rainout::available_midi_backends());
    }

    fn update(&mut self, ctx: &egui::CtxRef, _frame: &epi::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Hello World!");
        });
    }
}
