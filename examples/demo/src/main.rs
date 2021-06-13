use eframe::{egui, epi};
use egui::ScrollArea;
use ringbuf::{Consumer, Producer, RingBuffer};

use rusty_daw_io::{
    ConfigStatus, FatalErrorHandler, FatalStreamError, ProcessInfo, RtProcessHandler, StreamInfo,
    SystemOptions,
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
    system_opts: SystemOptions,
    settings_tab: SettingsTab,

    status_msg: String,
    status_msg_open: bool,
    /*
    audio_engine_running: bool,

    stream_handle: Option<StreamHandle<MyRtProcessHandler, MyFatalErrorHandler>>,
    error_signal_rx: Option<Consumer<FatalStreamError>>,
    */
}

impl epi::App for DemoApp {
    fn name(&self) -> &str {
        "rusty-daw-io demo"
    }

    fn update(&mut self, ctx: &egui::CtxRef, _frame: &mut epi::Frame<'_>) {
        /*
        if let Some(messages) = &self.messages {
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

                // Alert the helper of the crash so it can try and recover the config from file later.
                config_state.audio_engine_just_crashed = true;

                config_state.do_refresh_audio_servers = true;
                config_state.do_refresh_midi_servers = true;

                *audio_engine_running = false;
                *stream_handle = None; // Settings this to `None` will cause the stream to cleanup and shutdown.

                // Show the status message to the user as a pop-up window.
                *status_msg = format!("Fatal stream error occurred: {}", error);
                *status_msg_open = true;

                // Drop the error signal receiver here.
            } else {
                // Keep the error signal receiver if there was no error.
                *error_signal_rx = Some(error_rx);
            }
        }
        */

        /*
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

                                    // Alert the helper that we started the stream so it can save the config to file.
                                    config_state.audio_engine_just_started = true;
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
        */

        egui::SidePanel::left("side_panel", 200.0).show(ctx, |ui| {
            ui.heading("Settings");

            ui.separator();
            ui.vertical_centered_justified(|ui| {
                ui.selectable_value(&mut self.settings_tab, SettingsTab::Audio, "Audio");
                ui.selectable_value(&mut self.settings_tab, SettingsTab::Midi, "Midi");
            });
        });

        let settings_tab = self.settings_tab;
        egui::CentralPanel::default().show(ctx, |ui| {
            //ui.set_enabled(!*audio_engine_running);

            match settings_tab {
                SettingsTab::Audio => self.audio_settings(ui),
                SettingsTab::Midi => self.midi_settings(ui),
            }
        });

        /*
        if *status_msg_open {
            egui::Window::new("Status").show(ctx, |ui| {
                ui.label(status_msg.as_str());

                if ui.button("Ok").clicked() {
                    *status_msg_open = false;
                }
            });
        }
        */
    }
}

impl DemoApp {
    pub fn new() -> Self {
        Self {
            system_opts: SystemOptions::new(),
            settings_tab: SettingsTab::Audio,
            status_msg: String::new(),
            status_msg_open: false,
        }
    }

    fn audio_settings(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("Audio Devices");

            if ui.button("Refresh").clicked() {
                self.system_opts.refresh_servers();
            }

            // Can't figure out how to right-align elements in egui. Use spacing as a hacky
            // way to mimic this.
            ui.add_space(150.0);
        });

        ui.separator();

        ScrollArea::auto_sized().show(ui, |ui| {
            ui.add_space(SPACING / 2.0);

            egui::Grid::new("audio_settings_grid")
                .striped(true)
                .spacing([50.0, 8.0])
                .show(ui, |ui| {
                    // Audio server (driver model)

                    ui.label("Driver Model");
                    let mut audio_server_selection =
                        self.system_opts.display_state().current_audio_server_index;
                    egui::ComboBox::from_id_source("driver_model")
                        .selected_text(&self.system_opts.display_state().current_audio_server_name)
                        .show_ui(ui, |ui| {
                            for (i, option) in self
                                .system_opts
                                .display_state()
                                .audio_server_options
                                .iter()
                                .enumerate()
                            {
                                ui.selectable_value(&mut audio_server_selection, i, option);
                            }
                        });
                    if audio_server_selection
                        != self.system_opts.display_state().current_audio_server_index
                    {
                        self.system_opts.select_audio_server(audio_server_selection);
                    }
                    ui.end_row();

                    // Audio device

                    ui.label("Audio Device");
                    if self.system_opts.display_state().audio_device_options.len() < 2 {
                        // Don't display a combo box if audio device is not configurable.
                        ui.label(&self.system_opts.display_state().current_audio_device_name);
                    } else {
                        let mut audio_device_selection =
                            self.system_opts.display_state().current_audio_device_index;
                        egui::ComboBox::from_id_source("audio_device")
                            .selected_text(
                                &self.system_opts.display_state().current_audio_device_name,
                            )
                            .show_ui(ui, |ui| {
                                for (i, option) in self
                                    .system_opts
                                    .display_state()
                                    .audio_device_options
                                    .iter()
                                    .enumerate()
                                {
                                    ui.selectable_value(&mut audio_device_selection, i, option);
                                }
                            });
                        if audio_device_selection
                            != self.system_opts.display_state().current_audio_device_index
                        {
                            self.system_opts.select_audio_device(audio_device_selection);
                        }
                    }
                    ui.end_row();

                    // Sample rate

                    ui.label("Sample Rate");
                    if self.system_opts.display_state().sample_rate_options.len() < 2 {
                        // Don't display a combo box if sample rate is not configurable.
                        ui.label(&self.system_opts.display_state().current_sample_rate_str);
                    } else {
                        let mut sample_rate_selection =
                            self.system_opts.display_state().current_sample_rate_index;
                        egui::ComboBox::from_id_source("sample_rate")
                            .selected_text(
                                &self.system_opts.display_state().current_sample_rate_str,
                            )
                            .show_ui(ui, |ui| {
                                for (i, option) in self
                                    .system_opts
                                    .display_state()
                                    .sample_rate_options
                                    .iter()
                                    .enumerate()
                                {
                                    ui.selectable_value(&mut sample_rate_selection, i, option);
                                }
                            });
                        if sample_rate_selection
                            != self.system_opts.display_state().current_sample_rate_index
                        {
                            self.system_opts.select_sample_rate(sample_rate_selection);
                        }
                    }
                    ui.end_row();

                    // Buffer size

                    ui.label("Buffer Size");
                    let min = self.system_opts.display_state().buffer_size_range.min;
                    let max = self.system_opts.display_state().buffer_size_range.max;
                    if min == max {
                        // Don't display a slider if buffer size is not configurable.
                        ui.label(&self.system_opts.display_state().current_buffer_size_str);
                    } else {
                        let mut selected_buffer_size =
                            self.system_opts.display_state().current_buffer_size;
                        if ui
                            .add(egui::Slider::new(&mut selected_buffer_size, min..=max))
                            .changed()
                        {
                            self.system_opts.select_buffer_size(selected_buffer_size);
                        };
                        if ui.button("Auto").clicked() {
                            self.system_opts.select_auto_buffer_size();
                        }
                    }
                    ui.end_row();
                });

            ui.separator();

            match self.system_opts.config_status() {
                ConfigStatus::Ok {
                    config,
                    sample_rate,
                    latency_frames,
                    latency_ms,
                } => {
                    ui.label(format!(
                        "sample rate: {}  |  latency: {} frames ({:.1} ms)",
                        sample_rate, latency_frames, latency_ms
                    ));
                }
                ConfigStatus::AudioServerUnavailable(server) => {
                    ui.label(format!(
                        "Cannot start audio engine: Audio server {} is unavailable",
                        server
                    ));
                }
                ConfigStatus::NoAudioDeviceAvailable => {
                    ui.label("Cannot start audio engine: No audio device is available");
                }
                ConfigStatus::UnknownError => {
                    ui.label("Cannot start audio engine: Unkown error | Please check system logs");
                }
            }

            ui.add_space(SPACING);

            ui.heading("Output Busses");

            ui.separator();

            // Play nicely with borrow checker and egui's closures.
            let mut name_changes: Vec<(usize, String)> = Vec::new();
            let mut remove_busses: Vec<usize> = Vec::new();
            let mut system_port_changes: Vec<(usize, usize, usize)> = Vec::new();
            let mut remove_ports: Vec<(usize, usize)> = Vec::new();
            let mut add_new_port: Vec<usize> = Vec::new();
            for (bus_i, bus) in self
                .system_opts
                .display_state()
                .audio_out_busses
                .iter()
                .enumerate()
            {
                // egui requires a unique id for each grid
                let grid_id = format!("out_bus_grid_{}", bus_i);
                egui::Grid::new(grid_id)
                    .striped(true)
                    .spacing([50.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Name");
                        let mut bus_name = bus.id.clone();
                        if ui
                            .add(egui::TextEdit::singleline(&mut bus_name).hint_text("Enter Name"))
                            .changed()
                        {
                            name_changes.push((bus_i, bus_name));
                        };
                        if ui
                            .add(egui::Button::new("Remove Bus").enabled(bus.can_remove))
                            .clicked()
                        {
                            remove_busses.push(bus_i);
                        }
                        ui.end_row();

                        for (port_i, port) in bus.ports.iter().enumerate() {
                            ui.label(format!("port #{}", port_i + 1));

                            let mut system_port_selection = port.current_system_port_index;

                            // egui requires a unique id for each combo box
                            let cb_id = format!("out_bus_{}_port_{}", bus_i, port_i);
                            egui::ComboBox::from_id_source(cb_id)
                                .selected_text(&port.current_system_port_name)
                                .show_ui(ui, |ui| {
                                    for (i, option) in self
                                        .system_opts
                                        .display_state()
                                        .audio_out_system_port_options
                                        .iter()
                                        .enumerate()
                                    {
                                        if ui
                                            .selectable_value(&mut system_port_selection, i, option)
                                            .changed()
                                        {
                                            system_port_changes.push((
                                                bus_i,
                                                port_i,
                                                system_port_selection,
                                            ));
                                        };
                                    }
                                });

                            if ui
                                .add(egui::Button::new("x").enabled(port.can_remove))
                                .clicked()
                            {
                                remove_ports.push((bus_i, port_i));
                            }

                            ui.end_row();
                        }

                        if ui.button("Add Port").clicked() {
                            add_new_port.push(bus_i);
                        }
                    });

                ui.separator();
            }
            if ui.button("Add Bus").clicked() {
                self.system_opts.add_audio_out_bus();
            }
            for (bus_i, new_name) in name_changes.iter() {
                self.system_opts.rename_audio_out_bus(*bus_i, new_name);
            }
            for bus_i in remove_busses.iter() {
                self.system_opts.remove_audio_out_bus(*bus_i);
            }
            for (bus_i, port_i, new_system_port) in system_port_changes.iter() {
                self.system_opts.select_audio_out_bus_system_port(
                    *bus_i,
                    *port_i,
                    *new_system_port,
                );
            }
            for (bus_i, port_i) in remove_ports {
                self.system_opts.remove_audio_out_bus_port(bus_i, port_i);
            }
            for bus_i in add_new_port.iter() {
                self.system_opts.add_audio_out_bus_port(*bus_i);
            }

            ui.add_space(SPACING);

            ui.heading("Input Busses");

            ui.separator();

            if self.system_opts.display_state().playback_only {
                ui.label("(Playback only)");
            } else {
                // Play nicely with borrow checker and egui's closures.
                let mut name_changes: Vec<(usize, String)> = Vec::new();
                let mut remove_busses: Vec<usize> = Vec::new();
                let mut system_port_changes: Vec<(usize, usize, usize)> = Vec::new();
                let mut remove_ports: Vec<(usize, usize)> = Vec::new();
                let mut add_new_port: Vec<usize> = Vec::new();
                for (bus_i, bus) in self
                    .system_opts
                    .display_state()
                    .audio_in_busses
                    .iter()
                    .enumerate()
                {
                    // egui requires a unique id for each grid
                    let grid_id = format!("in_bus_grid_{}", bus_i);
                    egui::Grid::new(grid_id)
                        .striped(true)
                        .spacing([50.0, 8.0])
                        .show(ui, |ui| {
                            ui.label("Name");
                            let mut bus_name = bus.id.clone();
                            if ui
                                .add(
                                    egui::TextEdit::singleline(&mut bus_name)
                                        .hint_text("Enter Name"),
                                )
                                .changed()
                            {
                                name_changes.push((bus_i, bus_name));
                            };
                            if ui
                                .add(egui::Button::new("Remove Bus").enabled(bus.can_remove))
                                .clicked()
                            {
                                remove_busses.push(bus_i);
                            }
                            ui.end_row();

                            for (port_i, port) in bus.ports.iter().enumerate() {
                                ui.label(format!("port #{}", port_i + 1));

                                let mut system_port_selection = port.current_system_port_index;

                                // egui requires a unique id for each combo box
                                let cb_id = format!("in_bus_{}_port_{}", bus_i, port_i);
                                egui::ComboBox::from_id_source(cb_id)
                                    .selected_text(&port.current_system_port_name)
                                    .show_ui(ui, |ui| {
                                        for (i, option) in self
                                            .system_opts
                                            .display_state()
                                            .audio_in_system_port_options
                                            .iter()
                                            .enumerate()
                                        {
                                            if ui
                                                .selectable_value(
                                                    &mut system_port_selection,
                                                    i,
                                                    option,
                                                )
                                                .changed()
                                            {
                                                system_port_changes.push((
                                                    bus_i,
                                                    port_i,
                                                    system_port_selection,
                                                ));
                                            };
                                        }
                                    });

                                if ui
                                    .add(egui::Button::new("x").enabled(port.can_remove))
                                    .clicked()
                                {
                                    remove_ports.push((bus_i, port_i));
                                }

                                ui.end_row();
                            }

                            if ui.button("Add Port").clicked() {
                                add_new_port.push(bus_i);
                            }
                        });

                    ui.separator();
                }
                if ui.button("Add Bus").clicked() {
                    self.system_opts.add_audio_in_bus();
                }
                for (bus_i, new_name) in name_changes.iter() {
                    self.system_opts.rename_audio_in_bus(*bus_i, new_name);
                }
                for bus_i in remove_busses.iter() {
                    self.system_opts.remove_audio_in_bus(*bus_i);
                }
                for (bus_i, port_i, new_system_port) in system_port_changes.iter() {
                    self.system_opts.select_audio_in_bus_system_port(
                        *bus_i,
                        *port_i,
                        *new_system_port,
                    );
                }
                for (bus_i, port_i) in remove_ports {
                    self.system_opts.remove_audio_in_bus_port(bus_i, port_i);
                }
                for bus_i in add_new_port.iter() {
                    self.system_opts.add_audio_in_bus_port(*bus_i);
                }
            }
        });
    }

    fn midi_settings(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("Midi Devices");

            if ui.button("Refresh").clicked() {
                self.system_opts.refresh_servers();
            }

            // Can't figure out how to right-align elements in egui. Use spacing as a hacky
            // way to mimic this.
            ui.add_space(150.0);
        });

        ui.separator();

        ScrollArea::auto_sized().show(ui, |ui| {
            ui.add_space(SPACING / 2.0);

            egui::Grid::new("midi_settings_grid")
                .striped(true)
                .spacing([50.0, 8.0])
                .show(ui, |ui| {
                    // Midi server (driver model)

                    ui.label("Driver Model");
                    let mut midi_server_selection =
                        self.system_opts.display_state().current_midi_server_index;
                    egui::ComboBox::from_id_source("driver_model")
                        .selected_text(&self.system_opts.display_state().current_midi_server_name)
                        .show_ui(ui, |ui| {
                            for (i, option) in self
                                .system_opts
                                .display_state()
                                .midi_server_options
                                .iter()
                                .enumerate()
                            {
                                ui.selectable_value(&mut midi_server_selection, i, option);
                            }
                        });
                    if midi_server_selection
                        != self.system_opts.display_state().current_midi_server_index
                    {
                        self.system_opts.select_midi_server(midi_server_selection);
                    }
                    ui.end_row();
                });

            ui.add_space(SPACING);

            ui.heading("Input Controllers");

            ui.separator();

            if self
                .system_opts
                .display_state()
                .midi_in_system_port_options
                .is_empty()
            {
                ui.label("(No midi in ports found)");
            } else {
                // Play nicely with borrow checker and egui's closures.
                let mut name_changes: Vec<(usize, String)> = Vec::new();
                let mut remove_controllers: Vec<usize> = Vec::new();
                let mut system_port_changes: Vec<(usize, usize)> = Vec::new();
                for (controller_i, controller) in self
                    .system_opts
                    .display_state()
                    .midi_in_controllers
                    .iter()
                    .enumerate()
                {
                    // egui requires a unique id for each grid
                    let grid_id = format!("in_controller_grid_{}", controller_i);
                    egui::Grid::new(grid_id)
                        .striped(true)
                        .spacing([50.0, 8.0])
                        .show(ui, |ui| {
                            ui.label("Name");
                            let mut controller_name = controller.id.clone();
                            if ui
                                .add(
                                    egui::TextEdit::singleline(&mut controller_name)
                                        .hint_text("Enter Name"),
                                )
                                .changed()
                            {
                                name_changes.push((controller_i, controller_name));
                            };
                            if ui.add(egui::Button::new("Remove")).clicked() {
                                remove_controllers.push(controller_i);
                            }
                            ui.end_row();

                            ui.label("port");

                            let mut system_port_selection =
                                controller.system_port.current_system_port_index;

                            // egui requires a unique id for each combo box
                            let cb_id = format!("in_controller_{}_port", controller_i);
                            egui::ComboBox::from_id_source(cb_id)
                                .selected_text(&controller.system_port.current_system_port_name)
                                .show_ui(ui, |ui| {
                                    for (i, option) in self
                                        .system_opts
                                        .display_state()
                                        .midi_in_system_port_options
                                        .iter()
                                        .enumerate()
                                    {
                                        if ui
                                            .selectable_value(&mut system_port_selection, i, option)
                                            .changed()
                                        {
                                            system_port_changes
                                                .push((controller_i, system_port_selection));
                                        };
                                    }
                                });
                            ui.end_row();
                        });

                    ui.separator();
                }
                if ui.button("Add Controller").clicked() {
                    self.system_opts.add_midi_in_controller();
                }
                for (controller_i, new_name) in name_changes.iter() {
                    self.system_opts
                        .rename_midi_in_controller(*controller_i, new_name);
                }
                for controller_i in remove_controllers.iter() {
                    self.system_opts.remove_midi_in_controller(*controller_i);
                }
                for (controller_i, new_system_port) in system_port_changes.iter() {
                    self.system_opts
                        .select_midi_in_controller_system_port(*controller_i, *new_system_port);
                }
            }

            ui.add_space(SPACING);

            ui.heading("Output Controllers");

            ui.separator();

            if self
                .system_opts
                .display_state()
                .midi_out_system_port_options
                .is_empty()
            {
                ui.label("(No midi out ports found)");
            } else {
                // Play nicely with borrow checker and egui's closures.
                let mut name_changes: Vec<(usize, String)> = Vec::new();
                let mut remove_controllers: Vec<usize> = Vec::new();
                let mut system_port_changes: Vec<(usize, usize)> = Vec::new();
                for (controller_i, controller) in self
                    .system_opts
                    .display_state()
                    .midi_out_controllers
                    .iter()
                    .enumerate()
                {
                    // egui requires a unique id for each grid
                    let grid_id = format!("out_controller_grid_{}", controller_i);
                    egui::Grid::new(grid_id)
                        .striped(true)
                        .spacing([50.0, 8.0])
                        .show(ui, |ui| {
                            ui.label("Name");
                            let mut controller_name = controller.id.clone();
                            if ui
                                .add(
                                    egui::TextEdit::singleline(&mut controller_name)
                                        .hint_text("Enter Name"),
                                )
                                .changed()
                            {
                                name_changes.push((controller_i, controller_name));
                            };
                            if ui.add(egui::Button::new("Remove")).clicked() {
                                remove_controllers.push(controller_i);
                            }
                            ui.end_row();

                            ui.label("port");

                            let mut system_port_selection =
                                controller.system_port.current_system_port_index;

                            // egui requires a unique id for each combo box
                            let cb_id = format!("out_controller_{}_port", controller_i);
                            egui::ComboBox::from_id_source(cb_id)
                                .selected_text(&controller.system_port.current_system_port_name)
                                .show_ui(ui, |ui| {
                                    for (i, option) in self
                                        .system_opts
                                        .display_state()
                                        .midi_out_system_port_options
                                        .iter()
                                        .enumerate()
                                    {
                                        if ui
                                            .selectable_value(&mut system_port_selection, i, option)
                                            .changed()
                                        {
                                            system_port_changes
                                                .push((controller_i, system_port_selection));
                                        };
                                    }
                                });
                            ui.end_row();
                        });

                    ui.separator();
                }
                if ui.button("Add Controller").clicked() {
                    self.system_opts.add_midi_out_controller();
                }
                for (controller_i, new_name) in name_changes.iter() {
                    self.system_opts
                        .rename_midi_out_controller(*controller_i, new_name);
                }
                for controller_i in remove_controllers.iter() {
                    self.system_opts.remove_midi_out_controller(*controller_i);
                }
                for (controller_i, new_system_port) in system_port_changes.iter() {
                    self.system_opts
                        .select_midi_out_controller_system_port(*controller_i, *new_system_port);
                }
            }
        });
    }
}
