use eframe::{egui, epi};

use rusty_daw_io::{BufferSizeSelection, DeviceIOConfigHelper, DeviceIOConfigState};

static SPACING: f32 = 30.0;

fn main() {
    simple_logger::SimpleLogger::new().init().unwrap();

    let app = DemoApp::new();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum SettingsTab {
    Audio,
    Midi,
}

pub struct DemoApp {
    config_state: DeviceIOConfigState,
    config_helper: DeviceIOConfigHelper,

    settings_tab: SettingsTab,
}

impl epi::App for DemoApp {
    fn name(&self) -> &str {
        "rusty-daw-io demo"
    }

    fn update(&mut self, ctx: &egui::CtxRef, _frame: &mut epi::Frame<'_>) {
        let Self {
            config_state,
            config_helper,
            settings_tab,
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

            ui.separator();
            ui.vertical_centered_justified(|ui| {
                ui.selectable_value(settings_tab, SettingsTab::Audio, "Audio");
                ui.selectable_value(settings_tab, SettingsTab::Midi, "Midi");
            });
        });

        let settings_tab = *settings_tab;
        egui::CentralPanel::default().show(ctx, |ui| match settings_tab {
            SettingsTab::Audio => self.audio_settings(ui),
            SettingsTab::Midi => self.midi_settings(ui),
        });
    }
}

impl DemoApp {
    pub fn new() -> Self {
        let (config_helper, config_state) = DeviceIOConfigHelper::new();

        Self {
            config_state,
            config_helper,

            settings_tab: SettingsTab::Audio,
        }
    }

    fn audio_settings(&mut self, ui: &mut egui::Ui) {
        let Self {
            config_state,
            config_helper,
            ..
        } = self;

        ui.horizontal(|ui| {
            ui.heading("Audio Devices");

            if ui.button("Refresh").clicked() {
                config_helper.refresh_audio_servers(config_state);
            }
        });

        ui.separator();

        ui.vertical(|ui| {
            ui.add_space(SPACING / 2.0);

            // Audio server (driver model)

            ui.heading("System Device");

            ui.separator();

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

            if let Some(audio_device_options) = config_helper.audio_server_device_options() {
                egui::ComboBox::from_label("Device")
                    .selected_text(&audio_device_options.options[audio_device_options.selected])
                    .show_ui(ui, |ui| {
                        for (i, option) in audio_device_options.options.iter().enumerate() {
                            ui.selectable_value(&mut config_state.audio_server_device, i, option);
                        }
                    });

                if config_helper.audio_server_device_playback_only() {
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

            if config_helper.audio_server_device_not_selected() {
                ui.label("Cannot start audio engine. No device is selected.");
            }

            if config_helper.audio_server_unavailable() {
                ui.label(format!(
                    "Cannot start audio engine. {} audio server is unavailable",
                    config_helper.audio_server_options().options
                        [config_helper.audio_server_options().selected]
                ));
            }

            ui.add_space(SPACING);

            // User Audio Output Busses

            if let Some((bus_configs, available_ports)) = config_helper.audio_out_bus_config() {
                ui.heading("Output Busses");

                ui.separator();

                for (bus_i, (bus, bus_state)) in bus_configs
                    .iter()
                    .zip(config_state.audio_out_busses.iter_mut())
                    .enumerate()
                {
                    ui.horizontal(|ui| {
                        ui.add(egui::TextEdit::singleline(&mut bus_state.id).hint_text(&bus.id));
                        ui.label("Name");

                        // Don't allow user to delete the only output bus.
                        if bus_configs.len() > 1 {
                            if ui.button("Remove").clicked() {
                                // Mark the device for deletion.
                                bus_state.do_delete = true;
                            }
                        }
                    });

                    for (port_i, (port, port_state)) in bus
                        .system_ports
                        .iter()
                        .zip(bus_state.system_ports.iter_mut())
                        .enumerate()
                    {
                        ui.horizontal(|ui| {
                            // egui requires a unique id for each combo box
                            let cb_id = format!("user_audio_out_bus_{}_{}", bus_i, port_i);

                            egui::ComboBox::from_id_source(cb_id)
                                .selected_text(&port)
                                .show_ui(ui, |ui| {
                                    for available_port in available_ports.iter() {
                                        ui.selectable_value(
                                            port_state,
                                            available_port.clone(),
                                            available_port,
                                        );
                                    }
                                });

                            // Don't allow user to delete the only port.
                            if bus.system_ports.len() > 1 {
                                if ui.small_button("x").clicked() {
                                    // Rename a port to "" to automatically delete the port.
                                    *port_state = String::from("");
                                }
                            }
                        });
                    }

                    if ui.button("Add Port").clicked() {
                        bus_state.system_ports.push(available_ports[0].clone());
                    }

                    ui.separator();
                }

                if ui.button("Add Output Bus").clicked() {
                    if let Some(new_bus) = config_helper.new_audio_out_bus() {
                        config_state.audio_out_busses.push(new_bus);
                    }
                }

                ui.separator();
            }

            ui.add_space(SPACING);

            // User Audio Input Busses

            if let Some((bus_configs, available_ports)) = config_helper.audio_in_bus_config() {
                ui.heading("Input Busses");

                ui.separator();

                for (bus_i, (bus, bus_state)) in bus_configs
                    .iter()
                    .zip(config_state.audio_in_busses.iter_mut())
                    .enumerate()
                {
                    ui.horizontal(|ui| {
                        ui.add(egui::TextEdit::singleline(&mut bus_state.id).hint_text(&bus.id));
                        ui.label("Name");

                        if ui.button("Remove").clicked() {
                            // Mark the bus for deletion.
                            bus_state.do_delete = true;
                        }
                    });

                    for (port_i, (port, port_state)) in bus
                        .system_ports
                        .iter()
                        .zip(bus_state.system_ports.iter_mut())
                        .enumerate()
                    {
                        ui.horizontal(|ui| {
                            // egui requires a unique id for each combo box
                            let cb_id = format!("user_audio_in_bus_{}_{}", bus_i, port_i);

                            egui::ComboBox::from_id_source(cb_id)
                                .selected_text(&port)
                                .show_ui(ui, |ui| {
                                    for available_port in available_ports.iter() {
                                        ui.selectable_value(
                                            port_state,
                                            available_port.clone(),
                                            available_port,
                                        );
                                    }
                                });

                            // Don't allow user to delete the only port.
                            if bus.system_ports.len() > 1 {
                                if ui.small_button("x").clicked() {
                                    // Rename a port to "" to automatically delete the port.
                                    *port_state = String::from("");
                                }
                            }
                        });
                    }

                    if ui.button("Add Port").clicked() {
                        bus_state.system_ports.push(available_ports[0].clone());
                    }

                    ui.separator();
                }

                if ui.button("Add Input Bus").clicked() {
                    if let Some(new_bus) = config_helper.new_audio_in_bus() {
                        config_state.audio_in_busses.push(new_bus);
                    }
                }

                ui.separator();
            }
        });
    }

    fn midi_settings(&mut self, ui: &mut egui::Ui) {
        let Self {
            config_state,
            config_helper,
            ..
        } = self;

        ui.horizontal(|ui| {
            ui.heading("Midi Devices");

            if ui.button("Refresh").clicked() {
                config_helper.refresh_midi_servers(config_state);
            }
        });

        ui.separator();

        ui.vertical(|ui| {
            // Midi server (driver model)

            let midi_server_options = config_helper.midi_server_options();
            egui::ComboBox::from_label("Driver Model")
                .selected_text(&midi_server_options.options[midi_server_options.selected])
                .show_ui(ui, |ui| {
                    for (i, option) in midi_server_options.options.iter().enumerate() {
                        ui.selectable_value(&mut config_state.midi_server, i, option);
                    }
                });

            ui.separator();

            // Error States

            if config_helper.midi_server_unavailable() {
                ui.label(format!(
                    "{} midi server is unavailable",
                    config_helper.midi_server_options().options
                        [config_helper.midi_server_options().selected]
                ));
            }

            ui.add_space(SPACING);

            // User MIDI Input Controllers

            ui.heading("Input Controllers");

            ui.separator();

            if let Some((controller_configs, available_ports)) =
                config_helper.midi_in_controller_config()
            {
                for (controller_i, (controller, controller_state)) in controller_configs
                    .iter()
                    .zip(config_state.midi_in_controllers.iter_mut())
                    .enumerate()
                {
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut controller_state.id)
                                .hint_text(&controller.id),
                        );
                        ui.label("Name");

                        if ui.button("Remove").clicked() {
                            // Mark the controller for deletion.
                            controller_state.do_delete = true;
                        }
                    });

                    ui.horizontal(|ui| {
                        // egui requires a unique id for each combo box
                        let cb_id = format!("user_midi_in_controller_{}", controller_i);

                        egui::ComboBox::from_id_source(cb_id)
                            .selected_text(&controller.system_port)
                            .show_ui(ui, |ui| {
                                for option in available_ports.iter() {
                                    ui.selectable_value(
                                        &mut controller_state.system_port,
                                        option.clone(),
                                        option,
                                    );
                                }
                            });

                        ui.label("System Port");
                    });

                    ui.separator();
                }

                if ui.button("Add Input Controller").clicked() {
                    if let Some(new_controller) = config_helper.new_midi_in_controller() {
                        config_state.midi_in_controllers.push(new_controller);
                    }
                }

                ui.separator();
            } else {
                ui.label("No MIDI input devices were found");

                ui.separator();
            }

            ui.add_space(SPACING);

            // User Audio Outputs

            ui.heading("Output Controllers");

            ui.separator();

            if let Some((controller_configs, available_ports)) =
                config_helper.midi_out_controller_config()
            {
                for (controller_i, (controller, controller_state)) in controller_configs
                    .iter()
                    .zip(config_state.midi_out_controllers.iter_mut())
                    .enumerate()
                {
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut controller_state.id)
                                .hint_text(&controller.id),
                        );
                        ui.label("Name");

                        if ui.button("Remove").clicked() {
                            // Mark the controller for deletion.
                            controller_state.do_delete = true;
                        }
                    });

                    ui.horizontal(|ui| {
                        // egui requires a unique id for each combo box
                        let cb_id = format!("user_midi_out_controller_{}", controller_i);

                        egui::ComboBox::from_id_source(cb_id)
                            .selected_text(&controller.system_port)
                            .show_ui(ui, |ui| {
                                for option in available_ports.iter() {
                                    ui.selectable_value(
                                        &mut controller_state.system_port,
                                        option.clone(),
                                        option,
                                    );
                                }
                            });

                        ui.label("System Port");
                    });

                    ui.separator();
                }

                if ui.button("Add Output Controller").clicked() {
                    if let Some(new_controller) = config_helper.new_midi_out_controller() {
                        config_state.midi_out_controllers.push(new_controller);
                    }
                }

                ui.separator();
            } else {
                ui.label("No MIDI output devices were found");

                ui.separator();
            }
        });
    }
}
