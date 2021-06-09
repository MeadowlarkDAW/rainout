use eframe::{egui, epi};
use egui::ScrollArea;
use ringbuf::{Consumer, Producer, RingBuffer};

use rusty_daw_io::{
    BufferSizeOptions, DeviceIOHelper, DeviceIOHelperFeedback, DeviceIOHelperState,
    FatalErrorHandler, FatalStreamError, ProcessInfo, RtProcessHandler, StreamHandle, StreamInfo,
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

struct MyRtProcessHandler {}

impl RtProcessHandler for MyRtProcessHandler {
    fn init(&mut self, stream_info: &StreamInfo) {}
    fn process(&mut self, proc_info: ProcessInfo) {}
}

struct MyFatalErrorHandler {
    error_signal_tx: Producer<FatalStreamError>,
}

impl FatalErrorHandler for MyFatalErrorHandler {
    fn fatal_stream_error(mut self, error: FatalStreamError) {
        self.error_signal_tx.push(error).unwrap();
    }
}

pub struct DemoApp {
    config_feedback: DeviceIOHelper,
    settings_tab: SettingsTab,

    status_msg: String,
    status_msg_open: bool,

    audio_engine_running: bool,

    stream_handle: Option<StreamHandle<MyRtProcessHandler, MyFatalErrorHandler>>,
    error_signal_rx: Option<Consumer<FatalStreamError>>,
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
            audio_engine_running,
            stream_handle,
            error_signal_rx,
        } = self;

        let (config_state, config_feedback, messages) = config_feedback.update();

        if let Some(messages) = &messages {
            status_msg.clear();
            for message in messages.iter() {
                status_msg.push_str(message.as_str());
                status_msg.push_str("\n\n");
            }
            *status_msg_open = true;
        }

        if let Some(mut error_rx) = error_signal_rx.take() {
            if let Some(error) = error_rx.pop() {
                // Fatal stream error occurred. Stop the stream and refresh servers.
                eprintln!("Fatal stream error: {}", error);

                *audio_engine_running = false;
                *stream_handle = None; // Settings this to `None` will cause the stream to cleanup and shutdown.

                config_state.do_refresh_audio_servers = true;
                config_state.do_refresh_midi_servers = true;

                // Show the status message to the user as a pop-up window.
                *status_msg = format!("Fatal stream error occurred: {}", error);
                *status_msg_open = true;

                // Drop the error signal receiver here.
            } else {
                // Keep the error signal receiver if there was no error.
                *error_signal_rx = Some(error_rx);
            }
        }

        egui::TopPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.set_min_height(60.0);

                if let Some(audio_config) = config_feedback.audio_config() {
                    if *audio_engine_running {
                        if ui.button("Stop Audio Engine").clicked() {
                            *audio_engine_running = false;
                            *stream_handle = None; // Settings this to `None` will cause the stream to cleanup and shutdown.
                            *error_signal_rx = None;
                        }
                    } else {
                        if ui.button("Activate Audio Engine").clicked() {
                            let midi_config = config_feedback.midi_config();

                            // Create a channel for sending a "fatal stream error". Only one message can ever be sent.
                            let (new_error_signal_tx, new_error_signal_rx) =
                                RingBuffer::<FatalStreamError>::new(1).split();

                            let my_process_handler = MyRtProcessHandler {};
                            let my_fatal_error_hanlder = MyFatalErrorHandler {
                                error_signal_tx: new_error_signal_tx,
                            };

                            *error_signal_rx = Some(new_error_signal_rx);

                            match rusty_daw_io::spawn_rt_thread(
                                audio_config,
                                midi_config,
                                None,
                                my_process_handler,
                                my_fatal_error_hanlder,
                            ) {
                                Ok(handle) => {
                                    *stream_handle = Some(handle);
                                    *audio_engine_running = true;
                                }
                                Err(e) => {
                                    *status_msg = format!("Could not start audio engine: {}", e);

                                    // Show the status message to the user as a pop-up window.
                                    *status_msg_open = true;
                                }
                            }
                        }
                    }
                } else {
                    ui.add(egui::Button::new("Activate Audio Engine").enabled(false));
                }
            });
        });

        egui::SidePanel::left("side_panel", 200.0).show(ctx, |ui| {
            ui.heading("Settings");

            ui.separator();
            ui.vertical_centered_justified(|ui| {
                ui.selectable_value(settings_tab, SettingsTab::Audio, "Audio");
                ui.selectable_value(settings_tab, SettingsTab::Midi, "Midi");
            });
        });

        let settings_tab = *settings_tab;
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.set_enabled(!*audio_engine_running);

            match settings_tab {
                SettingsTab::Audio => audio_settings(
                    ui,
                    config_state,
                    config_feedback,
                    status_msg,
                    status_msg_open,
                ),
                SettingsTab::Midi => midi_settings(
                    ui,
                    config_state,
                    config_feedback,
                    status_msg,
                    status_msg_open,
                ),
            }
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
            audio_engine_running: false,
            stream_handle: None,
            error_signal_rx: None,
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
        ui.add_space(150.0);

        if let Some(audio_config) = config_feedback.audio_config() {
            if ui.button("Save Audio Config").clicked() {
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
            ui.add(egui::Button::new("Save Audio Config").enabled(false));
        }

        if ui.button("Load Audio Config").clicked() {
            // Just using the root directory and a default filename, but you can use the system's
            // file dialog instead.
            match load_audio_config_from_file("test_audio_config.xml") {
                Ok(new_config) => {
                    config_state.do_load_audio_config = Some(new_config);
                }
                Err(e) => {
                    *status_msg = format!("Error loading config: {}", e);
                    eprintln!("{}", status_msg);

                    // Show the status message to the user as a pop-up window.
                    *status_msg_open = true;
                }
            }
        }
    });

    ui.separator();

    ScrollArea::auto_sized().show(ui, |ui| {
        ui.add_space(SPACING / 2.0);

        // Audio server (driver model)

        ui.heading("System Device");

        ui.separator();

        egui::ComboBox::from_label("Driver Model")
            .selected_text(&config_feedback.audio_server_options()[config_state.audio_server_index])
            .show_ui(ui, |ui| {
                for (i, option) in config_feedback.audio_server_options().iter().enumerate() {
                    ui.selectable_value(&mut config_state.audio_server_index, i, option);
                }
            });

        ui.separator();

        // Audio device

        if let Some(audio_device_options) = config_feedback.audio_device_options() {
            egui::ComboBox::from_label("Device")
                .selected_text(&audio_device_options[config_state.audio_device_index])
                .show_ui(ui, |ui| {
                    for (i, option) in audio_device_options.iter().enumerate() {
                        ui.selectable_value(&mut config_state.audio_device_index, i, option);
                    }
                });

            if config_feedback.audio_device_playback_only() {
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

        if config_feedback.audio_device_not_selected() {
            ui.label("Cannot start audio engine. No device is selected.");
        }

        if config_feedback.audio_server_unavailable() {
            ui.label(format!(
                "Cannot start audio engine. {} audio server is unavailable",
                config_feedback.audio_server_options()[config_state.audio_server_index]
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
    status_msg: &mut String,
    status_msg_open: &mut bool,
) {
    ui.horizontal(|ui| {
        use rusty_daw_io::save_file::{load_midi_config_from_file, write_midi_config_to_file};

        ui.heading("Midi Devices");

        if ui.button("Refresh").clicked() {
            config_state.do_refresh_midi_servers = true;
        }

        // Can't figure out how to right-align elements in egui. Use spacing as a hacky
        // way to mimic this.
        ui.add_space(180.0);

        if let Some(midi_config) = config_feedback.midi_config() {
            if ui.button("Save Midi Config").clicked() {
                // Just using the root directory and a default filename, but you can use the system's
                // file dialog instead.
                match write_midi_config_to_file("test_midi_config.xml", midi_config) {
                    Ok(()) => {
                        *status_msg =
                            String::from("Successfully saved config to \"test_midi_config.xml\"");
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
            ui.add(egui::Button::new("Save Midi Config").enabled(false));
        }

        if ui.button("Load Midi Config").clicked() {
            // Just using the root directory and a default filename, but you can use the system's
            // file dialog instead.
            match load_midi_config_from_file("test_midi_config.xml") {
                Ok(new_config) => {
                    config_state.do_load_midi_config = Some(new_config);
                }
                Err(e) => {
                    *status_msg = format!("Error loading config: {}", e);
                    eprintln!("{}", status_msg);

                    // Show the status message to the user as a pop-up window.
                    *status_msg_open = true;
                }
            }
        }
    });

    ui.separator();

    ScrollArea::auto_sized().show(ui, |ui| {
        // Midi server (driver model)

        egui::ComboBox::from_label("Driver Model")
            .selected_text(&config_feedback.midi_server_options()[config_state.midi_server_index])
            .show_ui(ui, |ui| {
                for (i, option) in config_feedback.midi_server_options().iter().enumerate() {
                    ui.selectable_value(&mut config_state.midi_server_index, i, option);
                }
            });

        ui.separator();

        // Error States

        if config_feedback.midi_server_unavailable() {
            ui.label(format!(
                "{} midi server is unavailable",
                config_feedback.midi_server_options()[config_state.midi_server_index]
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
