use eframe::{egui, epi};

use rusty_daw_io::{BufferSizeSelection, DeviceIOConfigHelper, DeviceIOConfigState};

fn main() {
    let app = DemoApp::default();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}

pub struct DemoApp {
    config_state: DeviceIOConfigState,
    config_helper: DeviceIOConfigHelper,
}

impl Default for DemoApp {
    fn default() -> Self {
        Self {
            config_state: Default::default(),
            config_helper: Default::default(),
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

                let options = &config_helper.audio_server().options;
                egui::ComboBox::from_label("Driver Model")
                    .selected_text(&options[config_helper.audio_server().selected])
                    .show_ui(ui, |ui| {
                        for (i, option) in options.iter().enumerate() {
                            ui.selectable_value(&mut config_state.audio_server, i, option);
                        }
                    });

                ui.separator();

                if config_helper.current_server_available() {
                    // Duplex device

                    let options = &config_helper.audio_device().options;
                    // We can opt-out of showing the user available duplex devices if there is only one.
                    // Audio servers like Jack will only ever have one "duplex device".
                    if options.len() > 1 {
                        egui::ComboBox::from_label("Device")
                            .selected_text(&options[config_helper.audio_device().selected])
                            .show_ui(ui, |ui| {
                                for (i, option) in options.iter().enumerate() {
                                    ui.selectable_value(&mut config_state.audio_device, i, option);
                                }
                            });
                    }

                    // Sample rate

                    let options = &config_helper.sample_rate().options;
                    egui::ComboBox::from_label("Sample Rate")
                        .selected_text(&options[config_helper.sample_rate().selected])
                        .show_ui(ui, |ui| {
                            for (i, option) in options.iter().enumerate() {
                                ui.selectable_value(&mut config_state.sample_rate_index, i, option);
                            }
                        });

                    // Buffer size

                    match config_helper.buffer_size() {
                        BufferSizeSelection::UnknownSize => {
                            ui.label("(Device does not support constant buffer sizes)");
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

                    ui.separator();

                    if config_helper.audio_device_selected() {
                        if let Some(info) = config_helper.audio_config_info() {
                            ui.label(format!("Using sample rate: {}", info.sample_rate));
                            ui.label(format!(
                                "Estimated latency: {} frames ({:.1} ms)",
                                info.estimated_latency, info.estimated_latency_ms,
                            ));
                        } else {
                            ui.label("Cannot start audio engine because of an unkown error.");
                        }
                    } else {
                        ui.label("Cannot start audio engine. No device is selected.");
                    }
                } else {
                    ui.label(format!(
                        "{} audio server is unavailable",
                        config_helper.audio_server().options[config_helper.audio_server().selected]
                    ));
                }
            })
        });
    }
}
