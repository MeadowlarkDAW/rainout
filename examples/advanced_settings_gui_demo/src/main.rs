use eframe::{
    egui::{self, Ui},
    epi,
};

use rusty_daw_io::{
    available_audio_backends, available_midi_backends, AudioBackendInfo, AudioBackendStatus,
    AudioBufferSizeInfo, AudioDeviceInfo, MidiBackendInfo, ProcessHandler, ProcessInfo,
    SampleRateInfo, StreamHandle, StreamInfo,
};

fn main() {
    simple_logger::SimpleLogger::new().init().unwrap();

    let app = DemoApp::new();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}

#[derive(Debug, Clone)]
enum DemoAppState {
    AudioSettings(AudioSettingsState),
}

#[derive(Debug, Clone)]
enum AudioSettingsState {
    BackendNotRunning {
        selected_backend_i: usize,
        selected_backend_version: Option<String>,
    },
    NoAvailableDevices {
        selected_backend_i: usize,
        selected_backend_version: Option<String>,
    },
    UsingSystemWideDevice {
        selected_backend_i: usize,
        selected_backend_version: Option<String>,

        device_state: AudioSettingsDeviceState,
    },
    DeviceSelected {
        selected_backend_i: usize,
        selected_backend_version: Option<String>,

        audio_device_options: Vec<String>,
        selected_audio_device_i: Option<usize>,

        device_state: Option<AudioSettingsDeviceState>,
    },
}

#[derive(Debug, Clone)]
struct AudioSettingsDeviceState {
    sample_rate_unknown: bool,
    sample_rate_options: Vec<String>,
    selected_sample_rate_i: usize,

    buffer_size_unkown: bool,
    buffer_size_not_fixed: Option<(String, String)>,
    buffer_size_options: Vec<String>,
    selected_buffer_size_i: usize,

    audio_in_port_options: Vec<String>,
    audio_out_port_options: Vec<String>,

    configured_audio_in_ports: Vec<usize>,
    configured_audio_out_ports: Vec<usize>,
}

pub struct DemoApp {
    stream_handle: Option<StreamHandle<DemoAppProcessHandler>>,
    state: DemoAppState,
    selected_audio_backend_info: Option<AudioBackendInfo>,
}

impl DemoApp {
    pub fn new() -> Self {
        Self {
            stream_handle: None,
            state: DemoAppState::AudioSettings(AudioSettingsState::Initializing),
            selected_audio_backend_info: None,
        }
    }

    fn select_audio_backend(&mut self, backend_i: usize) {
        // Make sure the selected backend is within bounds.
        let backend_i = if backend_i >= rusty_daw_io::available_audio_backends().len() {
            0 // Default backend
        } else {
            backend_i
        };

        let backend_info = rusty_daw_io::enumerate_audio_backend(
            rusty_daw_io::available_audio_backends()[backend_i],
        )
        .unwrap();

        match &backend_info.status {
            AudioBackendStatus::NotRunning => {
                self.state = DemoAppState::AudioSettings(AudioSettingsState::BackendNotRunning {
                    selected_backend_i: backend_i,
                    selected_backend_version: backend_info.version.clone(),
                });
            }
            AudioBackendStatus::RunningButNoDevices => {
                self.state = DemoAppState::AudioSettings(AudioSettingsState::NoAvailableDevices {
                    selected_backend_i: backend_i,
                    selected_backend_version: backend_info.version.clone(),
                });
            }
            AudioBackendStatus::RunningWithSystemWideDevice(device_info) => {
                let device_state = Self::default_audio_device_state(0, &backend_info).unwrap();

                self.state =
                    DemoAppState::AudioSettings(AudioSettingsState::UsingSystemWideDevice {
                        selected_backend_i: backend_i,
                        selected_backend_version: backend_info.version.clone(),

                        device_state,
                    });
            }
            AudioBackendStatus::Running { default_i, devices } => {
                let audio_device_options = devices.iter().map(|d| d.id.name.clone()).collect();

                if let Some(default_i) = default_i {
                    let device_state =
                        Self::default_audio_device_state(*default_i, &backend_info).unwrap();

                    self.state = DemoAppState::AudioSettings(AudioSettingsState::DeviceSelected {
                        selected_backend_i: backend_i,
                        selected_backend_version: backend_info.version.clone(),

                        audio_device_options,
                        selected_audio_device_i: Some(*default_i),

                        device_state: Some(device_state),
                    });
                } else {
                    self.state = DemoAppState::AudioSettings(AudioSettingsState::DeviceSelected {
                        selected_backend_i: backend_i,
                        selected_backend_version: backend_info.version.clone(),

                        audio_device_options,
                        selected_audio_device_i: None,

                        device_state: None,
                    });
                }

                let device_state = Self::default_audio_device_state(0, &backend_info).unwrap();
            }
        }

        // Keep backend info around for future changes to audio device selections.
        self.selected_audio_backend_info = Some(backend_info);
    }

    fn select_audio_device(&mut self, device_i: Option<usize>) {
        if let DemoAppState::AudioSettings(AudioSettingsState::DeviceSelected {
            selected_audio_device_i,
            device_state,
            ..
        }) = &mut self.state
        {
            if let Some(device_i) = device_i {
                let backend_info = self.selected_audio_backend_info.as_ref().unwrap();
                let new_device_state =
                    Self::default_audio_device_state(device_i, backend_info).unwrap();

                *selected_audio_device_i = Some(device_i);
                *device_state = Some(new_device_state);
            } else {
                *selected_audio_device_i = None;
                *device_state = None;
            }
        }
    }

    fn default_audio_device_state(
        device_i: usize,
        backend_info: &AudioBackendInfo,
    ) -> Option<AudioSettingsDeviceState> {
        if let Some(audio_device_info) = backend_info.device_info(device_i) {
            let (sample_rate_unknown, sample_rate_options, selected_sample_rate_i) =
                match &audio_device_info.sample_rates {
                    SampleRateInfo::Unknown => (true, Vec::new(), 0),
                    SampleRateInfo::Unconfigurable(sample_rate) => (false, vec![*sample_rate], 0),
                    SampleRateInfo::List { options, default_i } => {
                        (false, options.clone(), *default_i)
                    }
                };

            let sample_rate_options =
                sample_rate_options.iter().map(|s| format!("{}", s)).collect();

            let (
                buffer_size_unkown,
                buffer_size_not_fixed,
                buffer_size_options,
                selected_buffer_size_i,
            ) = match &audio_device_info.buffer_sizes {
                AudioBufferSizeInfo::Unknown => (true, None, Vec::new(), 0),
                AudioBufferSizeInfo::UnconfigurableNotFixed { min, max } => {
                    (false, Some((*min, *max)), Vec::new(), 0)
                }
                AudioBufferSizeInfo::UnconfigurableFixed(buffer_size) => {
                    (false, None, vec![*buffer_size], 0)
                }
                AudioBufferSizeInfo::FixedList { options, default_i } => {
                    (false, None, options.clone(), *default_i)
                }
            };

            let buffer_size_options =
                buffer_size_options.iter().map(|b| format!("{}", b)).collect();

            let audio_in_port_options = audio_device_info.in_ports.clone();
            let audio_out_port_options = audio_device_info.out_ports.clone();

            let configured_audio_in_ports =
                audio_device_info.default_input_layout.device_ports.clone();
            let configured_audio_out_ports =
                audio_device_info.default_output_layout.device_ports.clone();

            Some(AudioSettingsDeviceState {
                sample_rate_unknown,
                sample_rate_options,
                selected_sample_rate_i,

                buffer_size_unkown,
                buffer_size_not_fixed,
                buffer_size_options,
                selected_buffer_size_i,

                audio_in_port_options,
                audio_out_port_options,

                configured_audio_in_ports,
                configured_audio_out_ports,
            })
        } else {
            None
        }
    }

    fn audio_backend_dropdown(
        &mut self,
        ui: &mut Ui,
        selected_backend_i: usize,
        selected_backend_version: &Option<String>,
    ) {
        let mut selected_i = selected_backend_i;
        ui.horizontal(|ui| {
            egui::ComboBox::from_label("Audio Backend")
                .selected_text(available_audio_backends()[selected_backend_i].as_str())
                .show_ui(ui, |ui| {
                    for (i, backend) in available_audio_backends().iter().enumerate() {
                        if ui.selectable_value(&mut selected_i, i, backend.as_str()).changed() {
                            self.select_audio_backend(selected_i);
                        }
                    }
                });

            if let Some(version) = selected_backend_version {
                ui.label(version);
            }
        });
    }

    fn audio_device_dropdown(
        &mut self,
        ui: &mut Ui,
        audio_device_options: &Vec<String>,
        selected_audio_device_i: Option<usize>,
    ) {
        let mut selected_i = selected_audio_device_i;

        let selected_text =
            if let Some(i) = selected_i { &audio_device_options[i] } else { "<none>" };

        egui::ComboBox::from_label("Audio Backend").selected_text(selected_text).show_ui(
            ui,
            |ui| {
                // Add a "<none>" option
                if ui.selectable_value(&mut selected_i, None, "none").changed() {
                    self.select_audio_device(selected_i);
                }

                for (i, device) in audio_device_options.iter().enumerate() {
                    if ui.selectable_value(&mut selected_i, Some(i), device).changed() {
                        self.select_audio_device(selected_i);
                    }
                }
            },
        );
    }

    fn audio_device_settings(&mut self, ui: &mut Ui, device_state: &AudioSettingsDeviceState) {
        /*
        AudioSettingsDeviceState {
            sample_rate_unknown,
            sample_rate_options,
            selected_sample_rate_i,

            buffer_size_unkown,
            buffer_size_not_fixed,
            buffer_size_options,
            selected_buffer_size_i,

            audio_in_port_options,
            audio_out_port_options,

            configured_audio_in_ports,
            configured_audio_out_ports,
        }*/

        if device_state.sample_rate_unknown {
            ui.label("unkown sample rate");
        } else if device_state.sample_rate_options.len() == 1 {
            // Just give the sample rate info to the user instead of showing
            // a drop-down.
            ui.horizontal(|ui| {
                ui.label("sample rate: ");
                ui.label(&device_state.sample_rate_options[device_state.selected_sample_rate_i]);
            });
        } else {
            egui::ComboBox::from_label("sample rate")
                .selected_text(
                    &device_state.sample_rate_options[device_state.selected_sample_rate_i],
                )
                .show_ui(ui, |ui| {
                    for (i, sample_rate) in device_state.sample_rate_options.iter().enumerate() {
                        if ui
                            .selectable_value(
                                &mut device_state.selected_sample_rate_i,
                                i,
                                sample_rate,
                            )
                            .changed()
                        {
                            // todo
                        }
                    }
                });
        }

        if device_state.buffer_size_unkown {
            ui.label("unkown buffer/block size");
        } else if let Some((min, max)) = &device_state.buffer_size_not_fixed {
            ui.horizontal(|ui| {
                ui.label("unfixed buffer/block size | min: ");
                ui.label(min);
                ui.label(", max: ");
                ui.label(max);
            });
        } else if device_state.buffer_size_options.len() == 1 {
            // Just give the buffer size info to the user instead of showing
            // a drop-down.
            ui.horizontal(|ui| {
                ui.label("buffer/block size: ");
                ui.label(&device_state.buffer_size_options[device_state.selected_buffer_size_i]);
            });
        } else {
            egui::ComboBox::from_label("buffer/block size")
                .selected_text(
                    &device_state.buffer_size_options[device_state.selected_buffer_size_i],
                )
                .show_ui(ui, |ui| {
                    for (i, buffer_size) in device_state.buffer_size_options.iter().enumerate() {
                        if ui
                            .selectable_value(
                                &mut device_state.selected_buffer_size_i,
                                i,
                                buffer_size,
                            )
                            .changed()
                        {
                            // todo
                        }
                    }
                });
        }
    }
}

impl epi::App for DemoApp {
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
        dbg!(rusty_daw_io::available_audio_backends());
        dbg!(rusty_daw_io::available_midi_backends());

        if let AudioBackendStatus::Running { devices, default_i } =
            self.selected_audio_backend_info.as_ref().unwrap().status
        {
            self.selected_audio_device_i = default_i;
            if let Some(default_i) = default_i {}
        }
    }

    fn update(&mut self, ctx: &egui::CtxRef, _frame: &epi::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            match &self.state {
                DemoAppState::AudioSettings(s) => match s {
                    AudioSettingsState::BackendNotRunning {
                        selected_backend_i,
                        selected_backend_version,
                    } => {
                        self.audio_backend_dropdown(
                            ui,
                            *selected_backend_i,
                            selected_backend_version,
                        );

                        ui.colored_label(egui::Rgba::RED, "Audio backend is not running");
                    }
                    AudioSettingsState::NoAvailableDevices {
                        selected_backend_i,
                        selected_backend_version,
                    } => {
                        self.audio_backend_dropdown(
                            ui,
                            *selected_backend_i,
                            selected_backend_version,
                        );

                        ui.colored_label(egui::Rgba::RED, "No available audio devices found");
                    }
                    AudioSettingsState::UsingSystemWideDevice {
                        selected_backend_i,
                        selected_backend_version,

                        device_state,
                    } => {
                        self.audio_backend_dropdown(
                            ui,
                            *selected_backend_i,
                            selected_backend_version,
                        );

                        self.audio_device_settings(ui, device_state);
                    }
                    AudioSettingsState::DeviceSelected {
                        selected_backend_i,
                        selected_backend_version,

                        audio_device_options,
                        selected_audio_device_i,

                        device_state,
                    } => {
                        self.audio_backend_dropdown(
                            ui,
                            *selected_backend_i,
                            selected_backend_version,
                        );
                        self.audio_device_dropdown(
                            ui,
                            audio_device_options,
                            *selected_audio_device_i,
                        );

                        if let Some(device_state) = device_state {
                            self.audio_device_settings(ui, device_state);
                        } else {
                            ui.label("No device selected");
                        }
                    }
                },
            }

            ui.separator();

            ui.label("Inputs");

            let mut remove_configured_port: Option<usize> = None;
            let mut try_change_port: Option<(usize, usize)> = None;
            let mut move_port_up: Option<usize> = None;
            let mut move_port_down: Option<usize> = None;
            let num_configured_ports = self.configured_audio_in_ports.len();
            for (i, selected_port) in self.configured_audio_in_ports.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    let down_enabled = i < (num_configured_ports - 1);
                    ui.add_enabled_ui(down_enabled, |ui| {
                        if ui.button("v").clicked() {
                            move_port_down = Some(i);
                        }
                    });

                    let up_enabled = i > 0;
                    ui.add_enabled_ui(up_enabled, |ui| {
                        if ui.button("^").clicked() {
                            move_port_up = Some(i);
                        }
                    });

                    egui::ComboBox::from_id_source(format!("audio_in_port_{}", i))
                        .selected_text(format!("{}", &audio_device_info.in_ports[*selected_port]))
                        .show_ui(ui, |ui| {
                            for (device_port_i, device_port_name) in
                                audio_device_info.in_ports.iter().enumerate()
                            {
                                let mut set_port = *selected_port;
                                if ui
                                    .selectable_value(
                                        &mut set_port,
                                        device_port_i,
                                        format!("{}", device_port_name),
                                    )
                                    .changed()
                                {
                                    try_change_port = Some((i, set_port));
                                };
                            }
                        });

                    if ui.button("X").clicked() {
                        remove_configured_port = Some(i);
                    }
                });
            }
            if let Some((i, new_port)) = try_change_port {
                if can_select_port(new_port, &self.configured_audio_in_ports) {
                    self.configured_audio_in_ports[i] = new_port;
                }
            }
            if let Some(i) = remove_configured_port {
                self.configured_audio_in_ports.remove(i);
            }
            if let Some(i) = move_port_up {
                self.configured_audio_in_ports.swap(i, i - 1);
            }
            if let Some(i) = move_port_down {
                self.configured_audio_in_ports.swap(i, i + 1);
            }

            let add_enabled =
                self.configured_audio_in_ports.len() < audio_device_info.in_ports.len();
            ui.add_enabled_ui(add_enabled, |ui| {
                if ui.button("Add Input").clicked() {
                    if let Some(new_port) = find_next_available_port(
                        &self.configured_audio_in_ports,
                        audio_device_info.in_ports.len(),
                    ) {
                        self.configured_audio_in_ports.push(new_port);
                    }
                }
            });

            ui.separator();

            ui.label("Outputs");

            let mut remove_configured_port: Option<usize> = None;
            let mut try_change_port: Option<(usize, usize)> = None;
            let mut move_port_up: Option<usize> = None;
            let mut move_port_down: Option<usize> = None;
            let num_configured_ports = self.configured_audio_out_ports.len();
            for (i, selected_port) in self.configured_audio_out_ports.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    let down_enabled = i < (num_configured_ports - 1);
                    ui.add_enabled_ui(down_enabled, |ui| {
                        if ui.button("v").clicked() {
                            move_port_down = Some(i);
                        }
                    });

                    let up_enabled = i > 0;
                    ui.add_enabled_ui(up_enabled, |ui| {
                        if ui.button("^").clicked() {
                            move_port_up = Some(i);
                        }
                    });

                    egui::ComboBox::from_id_source(format!("audio_out_port_{}", i))
                        .selected_text(format!("{}", &audio_device_info.out_ports[*selected_port]))
                        .show_ui(ui, |ui| {
                            for (device_port_i, device_port_name) in
                                audio_device_info.out_ports.iter().enumerate()
                            {
                                let mut set_port = *selected_port;
                                if ui
                                    .selectable_value(
                                        &mut set_port,
                                        device_port_i,
                                        format!("{}", device_port_name),
                                    )
                                    .changed()
                                {
                                    try_change_port = Some((i, set_port));
                                };
                            }
                        });

                    if ui.button("X").clicked() {
                        remove_configured_port = Some(i);
                    }
                });
            }
            if let Some((i, new_port)) = try_change_port {
                if can_select_port(new_port, &self.configured_audio_out_ports) {
                    self.configured_audio_out_ports[i] = new_port;
                }
            }
            if let Some(i) = remove_configured_port {
                self.configured_audio_out_ports.remove(i);
            }
            if let Some(i) = move_port_up {
                self.configured_audio_out_ports.swap(i, i - 1);
            }
            if let Some(i) = move_port_down {
                self.configured_audio_out_ports.swap(i, i + 1);
            }

            let add_enabled =
                self.configured_audio_out_ports.len() < audio_device_info.out_ports.len();
            ui.add_enabled_ui(add_enabled, |ui| {
                if ui.button("Add Output").clicked() {
                    if let Some(new_port) = find_next_available_port(
                        &self.configured_audio_out_ports,
                        audio_device_info.out_ports.len(),
                    ) {
                        self.configured_audio_out_ports.push(new_port);
                    }
                }
            });
        });
    }
}

fn can_select_port(new_port: usize, configured_ports: &[usize]) -> bool {
    !configured_ports.contains(&new_port)
}

fn find_next_available_port(
    configured_ports: &[usize],
    num_available_ports: usize,
) -> Option<usize> {
    if configured_ports.len() >= num_available_ports {
        return None;
    }

    for i in 0..num_available_ports {
        if !configured_ports.contains(&i) {
            return Some(i);
        }
    }

    None
}

struct DemoAppProcessHandler {}

impl ProcessHandler for DemoAppProcessHandler {
    /// Initialize/allocate any buffers here. This will only be called once on
    /// creation.
    fn init(&mut self, stream_info: &StreamInfo) {}

    /// This gets called if the user made a change to the configuration that does not
    /// require restarting the audio thread.
    fn stream_changed(&mut self, stream_info: &StreamInfo) {}

    /// Process the current buffers. This will always be called on a realtime thread.
    fn process<'a>(&mut self, proc_info: ProcessInfo<'a>) {}
}
