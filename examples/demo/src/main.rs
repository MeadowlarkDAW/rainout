use eframe::{egui, epi};
use egui::ScrollArea;

use rusty_daw_io::{
    BufferSizeOptions, DeviceIOHelper, DeviceIOHelperFeedback, DeviceIOHelperState,
};

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
    config_feedback: DeviceIOHelper,
    settings_tab: SettingsTab,

    status_msg: String,
    status_msg_open: bool,
}

impl epi::App for DemoApp {
    fn name(&self) -> &str {
        "rusty-daw-io demo"
    }

    fn update(&mut self, ctx: &egui::CtxRef, _frame: &mut epi::Frame<'_>) {
        let Self {
            config_feedback,
            settings_tab,
            status_msg,
            status_msg_open,
        } = self;

        let (config_state, config_feedback) = config_feedback.update();

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
            SettingsTab::Audio => audio_settings(
                ui,
                config_state,
                config_feedback,
                status_msg,
                status_msg_open,
            ),
            SettingsTab::Midi => midi_settings(ui, config_state, config_feedback),
        });

        if *status_msg_open {
            egui::Window::new("Status").show(ctx, |ui| {
                ui.label(status_msg.as_str());

                if ui.button("Ok").clicked() {
                    *status_msg_open = false;
                }
            });
        }
    }
}

impl DemoApp {
    pub fn new() -> Self {
        Self {
            config_feedback: Default::default(),
            settings_tab: SettingsTab::Audio,
            status_msg: String::new(),
            status_msg_open: false,
        }
    }
}

fn audio_settings(
    ui: &mut egui::Ui,
    config_state: &mut DeviceIOHelperState,
    config_feedback: &DeviceIOHelperFeedback,
    status_msg: &mut String,
    status_msg_open: &mut bool,
) {
    ui.horizontal(|ui| {
        use rusty_daw_io::save_file::{load_audio_config_from_file, write_audio_config_to_file};

        ui.heading("Audio Devices");

        if ui.button("Refresh").clicked() {
            config_state.do_refresh_audio_servers = true;
        }

        // Can't figure out how to right-align elements in egui. Use spacing as a hacky
        // way to mimic this.
        ui.add_space(225.0);

        if let Some(audio_config) = config_feedback.audio_server_config() {
            if ui.button("Save Config").clicked() {
                // Just using the root directory and a default filename, but you can use the system's
                // file dialog instead.
                match write_audio_config_to_file("test_audio_config.xml", audio_config) {
                    Ok(()) => {
                        *status_msg =
                            String::from("Successfully saved config to \"test_audio_config.xml\"");
                        println!("{}", status_msg);
                    }
                    Err(e) => {
                        *status_msg = format!("Error saving config: {}", e);
                        eprintln!("{}", status_msg);
                    }
                }

                // Show the status message to the user as a pop-up window.
                *status_msg_open = true;
            }
        } else {
            ui.add(egui::Button::new("Save Config").enabled(false));
        }

        if ui.button("Load Config").clicked() {
            // Just using the root directory and a default filename, but you can use the system's
            // file dialog instead.
            match load_audio_config_from_file("test_audio_config.xml") {
                Ok(new_config) => {
                    *status_msg =
                        String::from("Successfully loaded config from \"test_audio_config.xml\"");
                    println!("{}", status_msg);
                }
                Err(e) => {
                    *status_msg = format!("Error loading config: {}", e);
                    eprintln!("{}", status_msg);
                }
            }

            // Show the status message to the user as a pop-up window.
            *status_msg_open = true;
        }
    });

    ui.separator();

    ScrollArea::auto_sized().show(ui, |ui| {
        ui.add_space(SPACING / 2.0);

        // Audio server (driver model)

        ui.heading("System Device");

        ui.separator();

        egui::ComboBox::from_label("Driver Model")
            .selected_text(&config_feedback.audio_server_options()[config_state.audio_server])
            .show_ui(ui, |ui| {
                for (i, option) in config_feedback.audio_server_options().iter().enumerate() {
                    ui.selectable_value(&mut config_state.audio_server, i, option);
                }
            });

        ui.separator();

        // Audio device

        if let Some(audio_device_options) = config_feedback.audio_server_device_options() {
            egui::ComboBox::from_label("Device")
                .selected_text(&audio_device_options[config_state.audio_server_device])
                .show_ui(ui, |ui| {
                    for (i, option) in audio_device_options.iter().enumerate() {
                        ui.selectable_value(&mut config_state.audio_server_device, i, option);
                    }
                });

            if config_feedback.audio_server_device_playback_only() {
                ui.label("(Playback only)");
            }
        }

        // Sample rate

        if let Some(sample_rate_options) = config_feedback.sample_rate_options() {
            egui::ComboBox::from_label("Sample Rate")
                .selected_text(&sample_rate_options[config_state.sample_rate_index])
                .show_ui(ui, |ui| {
                    for (i, option) in sample_rate_options.iter().enumerate() {
                        ui.selectable_value(&mut config_state.sample_rate_index, i, option);
                    }
                });
        }

        // Buffer Size

        if let Some(buffer_size_options) = config_feedback.buffer_size_options() {
            match buffer_size_options {
                BufferSizeOptions::UnknownSize => {
                    ui.label("Unkown buffer size");
                }
                BufferSizeOptions::Constant { auto_value } => {
                    ui.label(format!("Buffer Size: {}", *auto_value));
                }
                BufferSizeOptions::Range {
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

        if let Some(info) = config_feedback.audio_config_info() {
            ui.label(format!("Using sample rate: {}", info.sample_rate));
            ui.label(format!(
                "Estimated latency: {} frames ({:.1} ms)",
                info.estimated_latency, info.estimated_latency_ms,
            ));
        }

        // Error States

        if config_feedback.audio_server_device_not_selected() {
            ui.label("Cannot start audio engine. No device is selected.");
        }

        if config_feedback.audio_server_unavailable() {
            ui.label(format!(
                "Cannot start audio engine. {} audio server is unavailable",
                config_feedback.audio_server_options()[config_state.audio_server]
            ));
        }

        ui.add_space(SPACING);

        // User Audio Output Busses

        if let Some(available_ports) = config_feedback.audio_out_port_options() {
            ui.heading("Output Busses");

            ui.separator();

            let num_out_busses = config_state.audio_out_busses.len();

            for (bus_i, bus_state) in config_state.audio_out_busses.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    ui.add(egui::TextEdit::singleline(&mut bus_state.id).hint_text("Enter Name"));
                    ui.label("Name");

                    // Don't allow user to delete the only output bus.
                    if num_out_busses > 1 {
                        if ui.button("Remove").clicked() {
                            // Mark the device for deletion.
                            bus_state.do_delete = true;
                        }
                    }
                });

                let num_system_ports = bus_state.system_ports.len();

                for (port_i, port_state) in bus_state.system_ports.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        // egui requires a unique id for each combo box
                        let cb_id = format!("user_audio_out_bus_{}_{}", bus_i, port_i);

                        egui::ComboBox::from_id_source(cb_id)
                            .selected_text(&port_state)
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
                        if num_system_ports > 1 {
                            if ui.small_button("x").clicked() {
                                // You may rename a port to "" to automatically delete the port.
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
                if let Some(new_bus) = config_feedback.new_audio_out_bus() {
                    config_state.audio_out_busses.push(new_bus);
                }
            }

            ui.separator();
        }

        ui.add_space(SPACING);

        // User Audio Input Busses

        if let Some(available_ports) = config_feedback.audio_in_port_options() {
            ui.heading("Input Busses");

            ui.separator();

            for (bus_i, bus_state) in config_state.audio_in_busses.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    ui.add(egui::TextEdit::singleline(&mut bus_state.id).hint_text("Enter Name"));
                    ui.label("Name");

                    if ui.button("Remove").clicked() {
                        // Mark the bus for deletion.
                        bus_state.do_delete = true;
                    }
                });

                let num_system_ports = bus_state.system_ports.len();

                for (port_i, port_state) in bus_state.system_ports.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        // egui requires a unique id for each combo box
                        let cb_id = format!("user_audio_in_bus_{}_{}", bus_i, port_i);

                        egui::ComboBox::from_id_source(cb_id)
                            .selected_text(&port_state)
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
                        if num_system_ports > 1 {
                            if ui.small_button("x").clicked() {
                                // You may rename a port to "" to automatically delete the port.
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
                if let Some(new_bus) = config_feedback.new_audio_in_bus() {
                    config_state.audio_in_busses.push(new_bus);
                }
            }

            ui.separator();
        }
    });
}

fn midi_settings(
    ui: &mut egui::Ui,
    config_state: &mut DeviceIOHelperState,
    config_feedback: &DeviceIOHelperFeedback,
) {
    ui.horizontal(|ui| {
        ui.heading("Midi Devices");

        if ui.button("Refresh").clicked() {
            config_state.do_refresh_midi_servers = true;
        }
    });

    ui.separator();

    ScrollArea::auto_sized().show(ui, |ui| {
        // Midi server (driver model)

        egui::ComboBox::from_label("Driver Model")
            .selected_text(&config_feedback.midi_server_options()[config_state.midi_server])
            .show_ui(ui, |ui| {
                for (i, option) in config_feedback.midi_server_options().iter().enumerate() {
                    ui.selectable_value(&mut config_state.midi_server, i, option);
                }
            });

        ui.separator();

        // Error States

        if config_feedback.midi_server_unavailable() {
            ui.label(format!(
                "{} midi server is unavailable",
                config_feedback.midi_server_options()[config_state.midi_server]
            ));
        }

        ui.add_space(SPACING);

        // User MIDI Input Controllers

        ui.heading("Input Controllers");

        ui.separator();

        if let Some(available_ports) = config_feedback.midi_in_port_options() {
            for (controller_i, controller_state) in
                config_state.midi_in_controllers.iter_mut().enumerate()
            {
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut controller_state.id)
                            .hint_text("Enter Name"),
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
                        .selected_text(&controller_state.system_port)
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
                if let Some(new_controller) = config_feedback.new_midi_in_controller() {
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

        if let Some(available_ports) = config_feedback.midi_out_port_options() {
            for (controller_i, controller_state) in
                config_state.midi_out_controllers.iter_mut().enumerate()
            {
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut controller_state.id)
                            .hint_text("Enter Name"),
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
                        .selected_text(&controller_state.system_port)
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
                if let Some(new_controller) = config_feedback.new_midi_out_controller() {
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
