use eframe::{egui, epi};

use rusty_daw_io::{BufferSizeSelection, DeviceIOConfigHelper, DeviceIOConfigState};

fn main() {
    let app = DemoApp::new();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}

pub struct DemoApp {
    config_state: DeviceIOConfigState,
    config_helper: DeviceIOConfigHelper,
}

impl DemoApp {
    pub fn new() -> Self {
        let (config_helper, config_state) = DeviceIOConfigHelper::new();

        Self {
            config_state,
            config_helper,
        }
    }
}

impl epi::App for DemoApp {
    fn name(&self) -> &str {
        "rusty-daw-io demo"
    }

    fn update(&mut self, ctx: &egui::CtxRef, _frame: &mut epi::Frame<'_>) {
        let Self {
            config_state,
            config_helper,
        } = self;

        config_helper.update(config_state);

        /*
        egui::TopPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                egui::menu::menu(ui, "File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });
            });
        });
        */

        egui::SidePanel::left("side_panel", 200.0).show(ctx, |ui| {
            ui.heading("Settings");

            /*
            ui.horizontal(|ui| {
                ui.label("Write something: ");
                ui.text_edit_singleline(label);
            });

            ui.add(egui::Slider::new(value, 0.0..=10.0).text("value"));
            if ui.button("Increment").clicked() {
                *value += 1.0;
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                ui.add(
                    egui::Hyperlink::new("https://github.com/emilk/egui/").text("powered by egui"),
                );
            });
            */
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's

            ui.heading("Audio Device");

            ui.separator();

            ui.vertical(|ui| {
                // Audio server (driver model)

                let audio_server_options = config_helper.audio_server_options();
                egui::ComboBox::from_label("Driver Model")
                    .selected_text(&audio_server_options.options[audio_server_options.selected])
                    .show_ui(ui, |ui| {
                        for (i, option) in audio_server_options.options.iter().enumerate() {
                            ui.selectable_value(&mut config_state.audio_server, i, option);
                        }
                    });

                ui.separator();

                // Audio device

                if let Some(audio_device_options) = config_helper.audio_device_options() {
                    egui::ComboBox::from_label("Device")
                        .selected_text(&audio_device_options.options[audio_device_options.selected])
                        .show_ui(ui, |ui| {
                            for (i, option) in audio_device_options.options.iter().enumerate() {
                                ui.selectable_value(&mut config_state.audio_device, i, option);
                            }
                        });

                    if config_helper.audio_device_playback_only() {
                        ui.label("(Playback only)");
                    }
                }

                // Sample rate

                if let Some(sample_rate_options) = config_helper.sample_rate_options() {
                    egui::ComboBox::from_label("Sample Rate")
                        .selected_text(&sample_rate_options.options[sample_rate_options.selected])
                        .show_ui(ui, |ui| {
                            for (i, option) in sample_rate_options.options.iter().enumerate() {
                                ui.selectable_value(&mut config_state.sample_rate_index, i, option);
                            }
                        });
                }

                // Buffer

                if let Some(buffer_size_options) = config_helper.buffer_size_options() {
                    match buffer_size_options {
                        BufferSizeSelection::UnknownSize => {
                            ui.label("Unkown buffer size");
                        }
                        BufferSizeSelection::Constant { auto_value } => {
                            ui.label(format!("Buffer Size: {}", *auto_value));
                        }
                        BufferSizeSelection::Range {
                            auto_value,
                            min,
                            max,
                            ..
                        } => {
                            ui.horizontal(|ui| {
                                if ui.button("Auto").clicked() {
                                    config_state.buffer_size = *auto_value;
                                }

                                ui.add(egui::Slider::new(
                                    &mut config_state.buffer_size,
                                    *min..=*max,
                                ));

                                ui.label("Buffer Size");
                            });
                        }
                    }
                }

                ui.separator();

                // Current Info

                if let Some(info) = config_helper.audio_config_info() {
                    ui.label(format!("Using sample rate: {}", info.sample_rate));
                    ui.label(format!(
                        "Estimated latency: {} frames ({:.1} ms)",
                        info.estimated_latency, info.estimated_latency_ms,
                    ));
                }

                // Error States

                if config_helper.audio_device_not_selected() {
                    ui.label("Cannot start audio engine. No device is selected.");
                }

                if config_helper.audio_server_unavailable() {
                    ui.label(format!(
                        "Cannot start audio engine. {} audio server is unavailable",
                        config_helper.audio_server_options().options
                            [config_helper.audio_server_options().selected]
                    ));
                }
            })
        });
    }
}
