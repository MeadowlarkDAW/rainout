use std::cell::RefCell;

use crate::{
    AudioBusConfig, AudioConfig, AudioServerDevices, AudioServerInfo, BufferSizeInfo, DevicesInfo,
    MidiConfig, MidiControllerConfig, MidiServerInfo, SystemDeviceInfo,
};

pub static DEFAULT_BUFFER_SIZE: u32 = 512;

#[derive(Debug, Clone, PartialEq)]
pub struct AudioBusConfigState {
    /// The ID to use for this bus. This ID is for the "internal" bus that appears to the user
    /// as list of available sources/sends. This is not necessarily the same as the name of the actual
    /// system hardware device that this "internal" bus is connected to.
    ///
    /// This ID *must* be unique for each `AudioBusConfig` and `MidiControllerConfig`.
    ///
    /// Examples of IDs can include:
    ///
    /// * Realtek Device In
    /// * Drums Mic
    /// * Headphones Out
    /// * Speakers Out
    pub id: String,

    /// The ports (of the system device) that this bus will be connected to.
    pub system_ports: Vec<String>,

    /// Whether or not this bus should be deleted.
    pub do_delete: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MidiControllerConfigState {
    /// The ID to use for this controller. This ID is for the "internal" controller that appears to the user
    /// as list of available sources/sends. This is not necessarily the same as the name of the actual
    /// system hardware device that this "internal" controller is connected to.
    ///
    /// This ID *must* be unique for each `AudioBusConfig` and `MidiControllerConfig`.
    pub id: String,

    /// The name of the system device port this controller is connected to.
    pub system_port: String,

    /// Whether or not this controller should be deleted.
    pub do_delete: bool,
}

#[derive(Debug, Clone)]
pub struct DeviceIOHelperState {
    pub audio_server_index: usize,
    pub audio_device_index: usize,

    pub midi_server_index: usize,

    pub sample_rate_index: usize,
    pub buffer_size: u32,

    pub audio_in_busses: Vec<AudioBusConfigState>,
    pub audio_out_busses: Vec<AudioBusConfigState>,

    pub midi_in_controllers: Vec<MidiControllerConfigState>,
    pub midi_out_controllers: Vec<MidiControllerConfigState>,

    pub do_refresh_audio_servers: bool,
    pub do_refresh_midi_servers: bool,

    pub do_load_audio_config: Option<AudioConfig>,
    pub do_load_midi_config: Option<MidiConfig>,
}

impl Default for DeviceIOHelperState {
    fn default() -> Self {
        Self {
            audio_server_index: 0,
            audio_device_index: 0,

            midi_server_index: 0,

            sample_rate_index: 0,
            buffer_size: DEFAULT_BUFFER_SIZE,

            audio_in_busses: Vec::new(),
            audio_out_busses: Vec::new(),

            midi_in_controllers: Vec::new(),
            midi_out_controllers: Vec::new(),

            do_refresh_audio_servers: false,
            do_refresh_midi_servers: false,

            do_load_audio_config: None,
            do_load_midi_config: None,
        }
    }
}

impl DeviceIOHelperState {
    fn update_from_audio_config(
        &mut self,
        new_config: AudioConfig,
        info: &[AudioServerInfo],
        messages: &mut Vec<String>,
    ) {
        let mut audio_server_exists = false;
        for (i, server) in info.iter().enumerate() {
            if &new_config.server == &server.name {
                audio_server_exists = true;
                self.audio_server_index = i;
                break;
            }
        }
        if !audio_server_exists {
            messages.push(format!(
                "Failed to load config: Server \"{}\" does not exist",
                &new_config.server
            ));
            return;
        }

        if let Some(audio_server_devices) = &info[self.audio_server_index].devices {
            let device = match &audio_server_devices {
                AudioServerDevices::SingleDevice(device) => {
                    // Ignore whatever exists in the new config and just use the only device.
                    self.audio_device_index = 1; // First device is "None".
                    device
                }
                AudioServerDevices::MultipleDevices(devices) => {
                    if devices.len() == 0 {
                        messages.push(format!(
                            "Failed to load config: Server \"{}\" has no available devices",
                            &new_config.server
                        ));
                        return;
                    }

                    let mut exists = false;
                    for (i, device) in devices.iter().enumerate() {
                        if &new_config.system_device == &device.name {
                            exists = true;
                            self.audio_device_index = i + 1; // First device is "None".
                            break;
                        }
                    }
                    if !exists {
                        // Select the first device and send a warning to the user.
                        self.audio_device_index = 1; // First device is "None".
                        messages.push(format!(
                            "Warning: System audio device \"{}\" could not be found",
                            &new_config.system_device
                        ));
                    }

                    &devices[self.audio_device_index - 1]
                }
            };

            if let Some(new_sample_rate) = new_config.sample_rate {
                let mut exists = false;
                for (i, sample_rate) in device.sample_rates.iter().enumerate() {
                    if new_sample_rate == *sample_rate {
                        exists = true;
                        self.sample_rate_index = i + 1; // First option is "Auto".
                        break;
                    }
                }
                if !exists {
                    // Select the option "Auto" and send a warning to the user.
                    self.sample_rate_index = 0;
                    messages.push(format!(
                        "Warning: Could not set sample rate to \"{}\"",
                        new_sample_rate
                    ));
                }
            } else {
                // Set the option to "Auto".
                self.sample_rate_index = 0;
            }

            if let Some(new_buffer_size) = new_config.buffer_size {
                match &device.buffer_size {
                    BufferSizeInfo::UnknownSize => {
                        // Ignore whatever exists in the new config and set to default.
                        self.buffer_size = DEFAULT_BUFFER_SIZE;
                    }
                    BufferSizeInfo::ConstantSize(size) => {
                        // Ignore whatever exists in the new config and use the only option.
                        self.buffer_size = *size;
                    }
                    &BufferSizeInfo::Range { min, max } => {
                        if new_buffer_size >= min && new_buffer_size <= max {
                            self.buffer_size = new_buffer_size;
                        } else {
                            // Set to the default buffer size and send a warning to the user.
                            self.buffer_size = DEFAULT_BUFFER_SIZE.min(min).max(max);
                            messages.push(format!(
                                "Warning: Could not set buffer size to \"{}\"",
                                new_buffer_size
                            ));
                        }
                    }
                }
            } else {
                // Set the option to default.
                self.buffer_size = DEFAULT_BUFFER_SIZE;
            }

            self.audio_in_busses.clear();
            self.audio_out_busses.clear();

            for new_bus in new_config.in_busses {
                if new_bus.system_ports.is_empty() {
                    // Ignore bus and send a warning to the user.
                    messages.push(format!(
                        "Warning: Could not create bus \"{}\". No ports given.",
                        &new_bus.id
                    ));
                    continue;
                }

                let mut failed_port = None;
                for port in new_bus.system_ports.iter() {
                    if !device.in_ports.contains(port) {
                        failed_port = Some(port.clone());
                        break;
                    }
                }
                if let Some(failed_port) = failed_port {
                    // Ignore bus and send a warning to the user.
                    messages.push(format!(
                        "Warning: Could not create bus \"{}\". The system port \"{}\" was not found.",
                        &new_bus.id, failed_port
                    ));
                    continue;
                }

                self.audio_in_busses.push(AudioBusConfigState {
                    id: new_bus.id.clone(),
                    system_ports: new_bus.system_ports.clone(),
                    do_delete: false,
                });
            }

            for new_bus in new_config.out_busses {
                if new_bus.system_ports.is_empty() {
                    // Ignore bus and send a warning to the user.
                    messages.push(format!(
                        "Warning: Could not create bus \"{}\". No ports given.",
                        &new_bus.id
                    ));
                    continue;
                }

                let mut failed_port = None;
                for port in new_bus.system_ports.iter() {
                    if !device.out_ports.contains(port) {
                        failed_port = Some(port.clone());
                        break;
                    }
                }
                if let Some(failed_port) = failed_port {
                    // Ignore bus and send a warning to the user.
                    messages.push(format!(
                        "Warning: Could not create bus \"{}\". The system port \"{}\" was not found.",
                        &new_bus.id, failed_port
                    ));
                    continue;
                }

                self.audio_out_busses.push(AudioBusConfigState {
                    id: new_bus.id.clone(),
                    system_ports: new_bus.system_ports.clone(),
                    do_delete: false,
                });
            }
        } else {
            messages.push(format!(
                "Failed to load config: Server \"{}\" is unavailable",
                &new_config.server
            ));
        }

        return;
    }

    fn update_from_midi_config(
        &mut self,
        new_config: MidiConfig,
        info: &[MidiServerInfo],
        messages: &mut Vec<String>,
    ) {
        let mut midi_server_exists = false;
        for (i, server) in info.iter().enumerate() {
            if &new_config.server == &server.name {
                midi_server_exists = true;
                self.midi_server_index = i;
                break;
            }
        }
        if !midi_server_exists {
            messages.push(format!(
                "Failed to load config: Server \"{}\" does not exist",
                &new_config.server
            ));
            return;
        }

        self.midi_in_controllers.clear();
        self.midi_out_controllers.clear();

        for new_controller in new_config.in_controllers {
            let mut exists = false;
            for device in info[self.midi_server_index].in_devices.iter() {
                if &device.name == &new_controller.system_port {
                    exists = true;
                    break;
                }
            }
            if !exists {
                // Ignore controller and send a warning to the user.
                messages.push(format!(
                    "Warning: Could not create controller \"{}\". The system port \"{}\" was not found.",
                    &new_controller.id, &new_controller.system_port
                ));
                continue;
            }

            self.midi_in_controllers.push(MidiControllerConfigState {
                id: new_controller.id.clone(),
                system_port: new_controller.system_port.clone(),
                do_delete: false,
            });
        }

        for new_controller in new_config.out_controllers {
            let mut exists = false;
            for device in info[self.midi_server_index].out_devices.iter() {
                if &device.name == &new_controller.system_port {
                    exists = true;
                    break;
                }
            }
            if !exists {
                // Ignore controller and send a warning to the user.
                messages.push(format!(
                    "Warning: Could not create controller \"{}\". The system port \"{}\" was not found.",
                    &new_controller.id, &new_controller.system_port
                ));
                continue;
            }

            self.midi_out_controllers.push(MidiControllerConfigState {
                id: new_controller.id.clone(),
                system_port: new_controller.system_port.clone(),
                do_delete: false,
            });
        }
    }
}

pub enum BufferSizeOptions {
    UnknownSize,
    Constant { auto_value: u32 },
    Range { auto_value: u32, min: u32, max: u32 },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AudioConfigInfo {
    pub estimated_latency: u32,
    pub estimated_latency_ms: f64,
    pub sample_rate: u32,
}

pub struct DeviceIOHelper {
    state: DeviceIOHelperState,
    feedback: DeviceIOHelperFeedback,
}

impl DeviceIOHelper {
    pub fn new() -> Self {
        let (feedback, state) = DeviceIOHelperFeedback::new();

        DeviceIOHelper { state, feedback }
    }

    pub fn update(
        &mut self,
    ) -> (
        &mut DeviceIOHelperState,
        &DeviceIOHelperFeedback,
        Option<Vec<String>>,
    ) {
        let mut did_refresh_audio_servers = false;
        let mut did_refresh_midi_servers = false;
        let mut auto_add_default_audio = true;
        let mut auto_add_default_midi = true;
        let mut messages = None;

        if self.state.do_refresh_audio_servers || self.state.do_refresh_midi_servers {
            if self.state.do_refresh_audio_servers {
                self.feedback.os_info.refresh_audio_servers();

                self.feedback.audio_server_options = self
                    .feedback
                    .os_info
                    .audio_servers_info()
                    .iter()
                    .map(|s| s.name.clone())
                    .collect();

                did_refresh_audio_servers = true;
                auto_add_default_audio = false;
            }

            if self.state.do_refresh_midi_servers {
                self.feedback.os_info.refresh_midi_servers();

                self.feedback.midi_server_options = self
                    .feedback
                    .os_info
                    .midi_servers_info()
                    .iter()
                    .map(|s| s.name.clone())
                    .collect();

                did_refresh_midi_servers = true;
                auto_add_default_midi = false;
            }

            self.state.do_refresh_audio_servers = false;
            self.state.do_refresh_midi_servers = false;
            self.feedback.state.do_refresh_audio_servers = false;
            self.feedback.state.do_refresh_midi_servers = false;
        }

        if self.state.do_load_audio_config.is_some() || self.state.do_load_midi_config.is_some() {
            let mut new_messages = Vec::<String>::new();

            if let Some(new_config) = self.state.do_load_audio_config.take() {
                self.state.update_from_audio_config(
                    new_config,
                    self.feedback.os_info.audio_servers_info(),
                    &mut new_messages,
                );

                auto_add_default_audio = false;
            }

            if let Some(new_config) = self.state.do_load_midi_config.take() {
                self.state.update_from_midi_config(
                    new_config,
                    self.feedback.os_info.midi_servers_info(),
                    &mut new_messages,
                );

                auto_add_default_midi = false;
            }

            // If no messages are present, then configs were loaded without errors.
            if !new_messages.is_empty() {
                messages = Some(new_messages)
            };
        }

        self.feedback.update(
            &mut self.state,
            did_refresh_audio_servers,
            did_refresh_midi_servers,
            auto_add_default_audio,
            auto_add_default_midi,
        );

        (&mut self.state, &self.feedback, messages)
    }
}

impl Default for DeviceIOHelper {
    fn default() -> Self {
        DeviceIOHelper::new()
    }
}

pub struct DeviceIOHelperFeedback {
    os_info: DevicesInfo,

    state: DeviceIOHelperState,

    audio_server_options: Vec<String>,
    audio_device_options: Option<Vec<String>>,

    midi_server_options: Vec<String>,

    sample_rate_options: Option<Vec<String>>,
    buffer_size_options: Option<BufferSizeOptions>,

    audio_config: Option<AudioConfig>,
    audio_config_info: Option<AudioConfigInfo>,

    midi_config: Option<MidiConfig>,

    midi_in_port_options: Vec<String>,
    midi_out_port_options: Vec<String>,

    // Interior mutability here is safe because these only count the number of
    // times a new bus/controller is successfully created from the user calling
    // `new_audio_in_bus()`, `new_audio_in_controller()`, etc.
    // This is so a unique default name can be always be returned for the new bus/controller.
    created_audio_in_busses: RefCell<usize>,
    created_audio_out_busses: RefCell<usize>,
    created_midi_in_controllers: RefCell<usize>,
    created_midi_out_controllers: RefCell<usize>,
}

impl DeviceIOHelperFeedback {
    fn new() -> (Self, DeviceIOHelperState) {
        let mut new_self = Self {
            os_info: DevicesInfo::new(),

            state: DeviceIOHelperState::default(),

            audio_server_options: Vec::new(),
            audio_device_options: None,

            midi_server_options: Vec::new(),

            sample_rate_options: None,
            buffer_size_options: None,

            audio_config: None,
            audio_config_info: None,

            midi_config: None,

            midi_in_port_options: Vec::new(),
            midi_out_port_options: Vec::new(),

            created_audio_in_busses: RefCell::new(0),
            created_audio_out_busses: RefCell::new(0),
            created_midi_in_controllers: RefCell::new(0),
            created_midi_out_controllers: RefCell::new(0),
        };

        new_self.audio_server_options = new_self
            .os_info
            .audio_servers_info()
            .iter()
            .map(|s| s.name.clone())
            .collect();

        new_self.midi_server_options = new_self
            .os_info
            .midi_servers_info()
            .iter()
            .map(|s| s.name.clone())
            .collect();

        let mut new_state = DeviceIOHelperState::default();

        new_self.update(&mut new_state, true, true, true, true);

        (new_self, new_state)
    }

    fn update(
        &mut self,
        new_state: &mut DeviceIOHelperState,
        did_refresh_audio_servers: bool,
        did_refresh_midi_servers: bool,
        auto_add_default_audio: bool,
        auto_add_default_midi: bool,
    ) -> bool {
        let mut audio_config_changed = false;
        let mut audio_device_changed = false;

        // Check if audio server selection changed

        if new_state.audio_server_index != self.state.audio_server_index
            || did_refresh_audio_servers
        {
            let index = new_state
                .audio_server_index
                .min(self.audio_server_options.len() - 1);
            new_state.audio_server_index = index;
            self.state.audio_server_index = index;

            // Rebuild available devices

            self.audio_device_options = if let Some(AudioServerDevices::MultipleDevices(devices)) =
                &self.os_info.audio_servers_info()[self.state.audio_server_index].devices
            {
                let mut options = vec![String::from("None")]; // Always have "None" as the first option.
                options.append(&mut devices.iter().map(|d| d.name.clone()).collect());

                Some(options)
            } else {
                None
            };

            self.state.audio_device_index = 0;

            audio_device_changed = true;
            audio_config_changed = true;
        }

        // Check if device selection changed

        if let Some(audio_device_options) = &mut self.audio_device_options {
            if new_state.audio_device_index != self.state.audio_device_index {
                let index = new_state
                    .audio_device_index
                    .min(audio_device_options.len() - 1);
                new_state.audio_device_index = index;
                self.state.audio_device_index = index;

                audio_device_changed = true;
                audio_config_changed = true;
            }
        }

        // Rebuild available options for device if changed

        if audio_device_changed {
            let (sample_rate_options, buffer_size_options) =
                if let Some(device) = self.current_audio_device_info() {
                    // Sample rate options
                    let mut sample_rate_options = vec![String::from("Auto")]; // Always have "Auto" as the first option for sample rate.
                    sample_rate_options.append(
                        &mut device
                            .sample_rates
                            .iter()
                            .map(|s| format!("{}", s))
                            .collect(),
                    );

                    let (buffer_size_options, value) = match &device.buffer_size {
                        BufferSizeInfo::ConstantSize(size) => {
                            (BufferSizeOptions::Constant { auto_value: *size }, *size)
                        }
                        BufferSizeInfo::Range { min, max } => (
                            BufferSizeOptions::Range {
                                auto_value: DEFAULT_BUFFER_SIZE.min(*min).max(*max),
                                min: *min,
                                max: *max,
                            },
                            DEFAULT_BUFFER_SIZE.min(*min).max(*max),
                        ),
                        BufferSizeInfo::UnknownSize => {
                            (BufferSizeOptions::UnknownSize, DEFAULT_BUFFER_SIZE)
                        }
                    };
                    self.state.buffer_size = value;

                    if auto_add_default_audio {
                        let (audio_in_busses, audio_out_busses) = self.default_audio_busses();
                        self.state.audio_in_busses = audio_in_busses.clone();
                        self.state.audio_out_busses = audio_out_busses.clone();
                        new_state.audio_in_busses = audio_in_busses;
                        new_state.audio_out_busses = audio_out_busses;
                    }

                    (Some(sample_rate_options), Some(buffer_size_options))
                } else {
                    self.buffer_size_options = None;
                    self.state.buffer_size = DEFAULT_BUFFER_SIZE;

                    self.state.audio_in_busses.clear();
                    self.state.audio_out_busses.clear();
                    new_state.audio_in_busses.clear();
                    new_state.audio_out_busses.clear();

                    (None, None)
                };

            self.sample_rate_options = sample_rate_options;
            self.buffer_size_options = buffer_size_options;
        }

        // Check if sample rate selection changed

        if let Some(sample_rate_options) = &mut self.sample_rate_options {
            if new_state.sample_rate_index != self.state.sample_rate_index {
                let index = new_state
                    .sample_rate_index
                    .min(sample_rate_options.len() - 1);
                new_state.sample_rate_index = index;
                self.state.sample_rate_index = index;

                audio_config_changed = true;
            }
        }

        // Check if buffer size selection changed

        if let Some(buffer_size_options) = &mut self.buffer_size_options {
            let new_size = match buffer_size_options {
                BufferSizeOptions::UnknownSize => {
                    DEFAULT_BUFFER_SIZE // Not that necessary, but make sure the user doesn't try to change this.
                }
                BufferSizeOptions::Constant { auto_value } => {
                    *auto_value // Make sure the user doesn't try to change this.
                }
                BufferSizeOptions::Range { min, max, .. } => {
                    if new_state.buffer_size != self.state.buffer_size {
                        audio_config_changed = true;
                        new_state.buffer_size.min(*min).max(*max)
                    } else {
                        new_state.buffer_size
                    }
                }
            };
            new_state.buffer_size = new_size;
            self.state.buffer_size = new_size;
        }

        // Check if audio busses have changed

        if self.state.audio_in_busses != new_state.audio_in_busses
            || self.state.audio_out_busses != new_state.audio_out_busses
        {
            let (audio_in_busses, mut audio_out_busses) =
                if let Some(device) = self.current_audio_device_info() {
                    let mut audio_in_busses = Vec::<AudioBusConfigState>::new();
                    let mut audio_out_busses = Vec::<AudioBusConfigState>::new();

                    // Delete any busses marked for deletion.
                    new_state.audio_in_busses.retain(|d| !d.do_delete);
                    new_state.audio_out_busses.retain(|d| !d.do_delete);

                    for bus in new_state.audio_in_busses.iter_mut() {
                        // Delete any ports with blank names.
                        bus.system_ports.retain(|p| p.len() > 0);

                        let mut all_ports_exist = true;
                        for port in bus.system_ports.iter() {
                            if !device.in_ports.contains(port) {
                                all_ports_exist = false;
                                break;
                            }
                        }

                        // Disregard bus if it is invalid.
                        if all_ports_exist && !bus.system_ports.is_empty() {
                            audio_in_busses.push(bus.clone());
                        }
                    }

                    for bus in new_state.audio_out_busses.iter_mut() {
                        // Delete any ports with blank names.
                        bus.system_ports.retain(|p| p.len() > 0);

                        let mut all_ports_exist = true;
                        for port in bus.system_ports.iter() {
                            if !device.out_ports.contains(port) {
                                all_ports_exist = false;
                                break;
                            }
                        }

                        // Disregard bus if it is invalid.
                        if all_ports_exist && !bus.system_ports.is_empty() {
                            audio_out_busses.push(bus.clone());
                        }
                    }

                    (audio_in_busses, audio_out_busses)
                } else {
                    (Vec::new(), Vec::new())
                };

            // Make sure there is always atleast one output bus.
            if audio_out_busses.is_empty() {
                let (_, audio_out) = self.default_audio_busses();
                audio_out_busses = audio_out;
            }

            self.state.audio_in_busses = audio_in_busses.clone();
            self.state.audio_out_busses = audio_out_busses.clone();
            new_state.audio_in_busses = audio_in_busses;
            new_state.audio_out_busses = audio_out_busses;

            audio_config_changed = true;
        }

        // Rebuild audio config if changed

        if audio_config_changed {
            self.audio_config = self.build_audio_config();

            // Get the current reported latency and sample rate

            if let Some(audio_config) = &self.audio_config {
                let estimated_latency = self.os_info.estimated_latency(audio_config).unwrap_or(0);
                let sample_rate = self.os_info.sample_rate(audio_config).unwrap_or(1);

                self.audio_config_info = Some(AudioConfigInfo {
                    estimated_latency,
                    sample_rate,
                    estimated_latency_ms: 1_000.0 * f64::from(estimated_latency)
                        / f64::from(sample_rate),
                })
            } else {
                self.audio_config_info = None;
            }
        }

        let mut midi_changed = false;

        // Check if midi server selection changed

        if new_state.midi_server_index != self.state.midi_server_index || did_refresh_midi_servers {
            let index = new_state
                .midi_server_index
                .min(self.midi_server_options.len() - 1);
            new_state.midi_server_index = index;
            self.state.midi_server_index = index;

            self.midi_in_port_options = self.os_info.midi_servers_info()
                [self.state.midi_server_index]
                .in_devices
                .iter()
                .map(|d| d.name.clone())
                .collect();
            self.midi_out_port_options = self.os_info.midi_servers_info()
                [self.state.midi_server_index]
                .out_devices
                .iter()
                .map(|d| d.name.clone())
                .collect();

            if auto_add_default_midi {
                let (midi_in_controllers, midi_out_controllers) = self.default_midi_controllers();
                self.state.midi_in_controllers = midi_in_controllers.clone();
                self.state.midi_out_controllers = midi_out_controllers.clone();
                new_state.midi_in_controllers = midi_in_controllers;
                new_state.midi_out_controllers = midi_out_controllers;
            }

            midi_changed = true;
        }

        // Check if midi controllers have changed

        if self.state.midi_in_controllers != new_state.midi_in_controllers
            || self.state.midi_out_controllers != new_state.midi_out_controllers
        {
            self.state.midi_in_controllers.clear();
            self.state.midi_out_controllers.clear();

            // Delete any controllers marked for deletion.
            new_state.midi_in_controllers.retain(|d| !d.do_delete);
            new_state.midi_out_controllers.retain(|d| !d.do_delete);

            for controller in new_state.midi_in_controllers.iter_mut() {
                let mut device_exists = false;
                for device in self.os_info.midi_servers_info()[self.state.midi_server_index]
                    .in_devices
                    .iter()
                {
                    if &controller.system_port == &device.name {
                        device_exists = true;
                        break;
                    }
                }

                // Disregard controller if it is invalid.
                if device_exists {
                    self.state.midi_in_controllers.push(controller.clone());
                }
            }

            for controller in new_state.midi_out_controllers.iter_mut() {
                let mut device_exists = false;
                for device in self.os_info.midi_servers_info()[self.state.midi_server_index]
                    .out_devices
                    .iter()
                {
                    if &controller.system_port == &device.name {
                        device_exists = true;
                        break;
                    }
                }

                // Disregard controller if it is invalid.
                if device_exists {
                    self.state.midi_out_controllers.push(controller.clone());
                }
            }

            // Make sure state matches.
            new_state.midi_in_controllers = self.state.midi_in_controllers.clone();
            new_state.midi_out_controllers = self.state.midi_out_controllers.clone();

            midi_changed = true;
        }

        // Rebuild midiconfig if changed

        if midi_changed {
            self.midi_config = self.build_midi_config();
        }

        audio_config_changed || midi_changed
    }

    pub fn new_audio_in_bus(&self) -> Option<AudioBusConfigState> {
        let new_bus = if let Some(device) = self.current_audio_device_info() {
            if device.in_ports.len() == 0 {
                None
            } else {
                // Find the index of the next available port.
                let mut next_port_i = 1;
                if let Some(last_bus) = self.state.audio_in_busses.last() {
                    if let Some(last_port) = last_bus.system_ports.last() {
                        for port in device.in_ports.iter() {
                            if port == last_port {
                                break;
                            }
                            next_port_i += 1;
                        }
                    }
                }
                if next_port_i >= device.in_ports.len() || self.state.audio_in_busses.is_empty() {
                    next_port_i = 0;
                }

                // If this is the first time calling this method and a bus was already created by default,
                // mark this new bus as the second one.
                if *self.created_audio_in_busses.borrow() == 0
                    && !self.state.audio_in_busses.is_empty()
                {
                    *self.created_audio_in_busses.borrow_mut() = 1;
                }

                Some(AudioBusConfigState {
                    id: format!("Mic #{}", *self.created_audio_in_busses.borrow() + 1),
                    system_ports: vec![device.in_ports[next_port_i].clone()],
                    do_delete: false,
                })
            }
        } else {
            None
        };

        if new_bus.is_some() {
            *self.created_audio_in_busses.borrow_mut() += 1;
        }

        new_bus
    }

    pub fn new_audio_out_bus(&self) -> Option<AudioBusConfigState> {
        let next_port = |device: &SystemDeviceInfo| {
            // Find the index of the next available port.
            let mut next_port_i = 1;
            if let Some(last_bus) = self.state.audio_out_busses.last() {
                if let Some(last_port) = last_bus.system_ports.last() {
                    for port in device.out_ports.iter() {
                        if port == last_port {
                            break;
                        }
                        next_port_i += 1;
                    }
                }
            }
            if next_port_i >= device.out_ports.len() || self.state.audio_out_busses.is_empty() {
                next_port_i = 0;
            }
            next_port_i
        };

        let new_bus = if let Some(device) = self.current_audio_device_info() {
            if device.out_ports.len() == 0 {
                None
            } else if device.out_ports.len() == 1 {
                let next_port_i = next_port(device);

                // If this is the first time calling this method and a bus was already created by default,
                // mark this new bus as the second one.
                if *self.created_audio_out_busses.borrow() == 0
                    && !self.state.audio_out_busses.is_empty()
                {
                    *self.created_audio_out_busses.borrow_mut() = 1;
                }

                Some(AudioBusConfigState {
                    id: format!(
                        "Mono Speaker #{}",
                        *self.created_audio_out_busses.borrow() + 1
                    ),
                    system_ports: vec![device.out_ports[next_port_i].clone()],
                    do_delete: false,
                })
            } else {
                let next_port_i = next_port(device);
                let second_port_i = if next_port_i + 1 >= device.out_ports.len() {
                    0
                } else {
                    next_port_i + 1
                };

                // If this is the first time calling this method and a bus was already created by default,
                // mark this new bus as the second one.
                if *self.created_audio_out_busses.borrow() == 0
                    && !self.state.audio_out_busses.is_empty()
                {
                    *self.created_audio_out_busses.borrow_mut() = 1;
                }

                Some(AudioBusConfigState {
                    id: format!(
                        "Stereo Speakers #{}",
                        *self.created_audio_out_busses.borrow() + 1
                    ),
                    system_ports: vec![
                        device.out_ports[next_port_i].clone(),
                        device.out_ports[second_port_i].clone(),
                    ],
                    do_delete: false,
                })
            }
        } else {
            None
        };

        if new_bus.is_some() {
            *self.created_audio_out_busses.borrow_mut() += 1;
        }

        new_bus
    }

    pub fn new_midi_in_controller(&self) -> Option<MidiControllerConfigState> {
        let new_controller = if !self.midi_in_port_options.is_empty() {
            // Find the index of the next available port.
            let mut next_port_i = 1;
            if let Some(last_controller) = self.state.midi_in_controllers.last() {
                for port in self.midi_in_port_options.iter() {
                    if port == &last_controller.system_port {
                        break;
                    }
                    next_port_i += 1;
                }
            }
            if next_port_i >= self.midi_in_port_options.len()
                || self.state.midi_in_controllers.is_empty()
            {
                next_port_i = 0;
            }

            // If this is the first time calling this method and a controller was already created by default,
            // mark this new controller as the second one.
            if *self.created_midi_in_controllers.borrow() == 0
                && !self.state.midi_in_controllers.is_empty()
            {
                *self.created_midi_in_controllers.borrow_mut() = 1;
            }

            Some(MidiControllerConfigState {
                id: format!(
                    "MIDI In Controller #{}",
                    *self.created_midi_in_controllers.borrow() + 1
                ),
                system_port: self.midi_in_port_options[next_port_i].clone(),
                do_delete: false,
            })
        } else {
            None
        };

        if new_controller.is_some() {
            *self.created_midi_in_controllers.borrow_mut() += 1;
        }

        new_controller
    }

    pub fn new_midi_out_controller(&self) -> Option<MidiControllerConfigState> {
        let new_controller = if !self.midi_out_port_options.is_empty() {
            // Find the index of the next available port.
            let mut next_port_i = 1;
            if let Some(last_controller) = self.state.midi_out_controllers.last() {
                for port in self.midi_out_port_options.iter() {
                    if port == &last_controller.system_port {
                        break;
                    }
                    next_port_i += 1;
                }
            }
            if next_port_i >= self.midi_out_port_options.len()
                || self.state.midi_out_controllers.is_empty()
            {
                next_port_i = 0;
            }

            // If this is the first time calling this method and a controller was already created by default,
            // mark this new controller as the second one.
            if *self.created_midi_out_controllers.borrow() == 0
                && !self.state.midi_out_controllers.is_empty()
            {
                *self.created_midi_out_controllers.borrow_mut() = 1;
            }

            Some(MidiControllerConfigState {
                id: format!(
                    "MIDI Out Controller #{}",
                    *self.created_midi_out_controllers.borrow() + 1,
                ),
                system_port: self.midi_out_port_options[next_port_i].clone(),
                do_delete: false,
            })
        } else {
            None
        };

        if new_controller.is_some() {
            *self.created_midi_out_controllers.borrow_mut() += 1;
        }

        new_controller
    }

    fn default_audio_busses(&self) -> (Vec<AudioBusConfigState>, Vec<AudioBusConfigState>) {
        let mut in_devices = Vec::<AudioBusConfigState>::new();
        let mut out_devices = Vec::<AudioBusConfigState>::new();

        if let Some(device) = self.current_audio_device_info() {
            if device.in_ports.len() > 0 {
                in_devices.push(AudioBusConfigState {
                    id: String::from("Mic #1"),
                    system_ports: vec![device.in_ports[0].clone()],
                    do_delete: false,
                })
            }

            if device.out_ports.len() == 1 {
                out_devices.push(AudioBusConfigState {
                    id: String::from("Mono Speaker"),
                    system_ports: vec![device.out_ports[0].clone()],
                    do_delete: false,
                });
            } else {
                out_devices.push(AudioBusConfigState {
                    id: String::from("Stereo Speakers"),
                    system_ports: vec![device.out_ports[0].clone(), device.out_ports[1].clone()],
                    do_delete: false,
                });
            }
        }

        (in_devices, out_devices)
    }

    fn default_midi_controllers(
        &self,
    ) -> (
        Vec<MidiControllerConfigState>,
        Vec<MidiControllerConfigState>,
    ) {
        let mut in_devices = Vec::<MidiControllerConfigState>::new();
        let out_devices = Vec::<MidiControllerConfigState>::new();

        // Only create a single midi input for now.

        if let Some(device) = self.os_info.midi_servers_info()[self.state.midi_server_index]
            .in_devices
            .first()
        {
            in_devices.push(MidiControllerConfigState {
                id: String::from("Midi In Controller"),
                system_port: device.name.clone(),
                do_delete: false,
            })
        }

        (in_devices, out_devices)
    }

    pub fn audio_server_options(&self) -> &Vec<String> {
        &self.audio_server_options
    }

    pub fn audio_device_options(&self) -> &Option<Vec<String>> {
        &self.audio_device_options
    }

    pub fn midi_server_options(&self) -> &Vec<String> {
        &self.midi_server_options
    }

    pub fn sample_rate_options(&self) -> &Option<Vec<String>> {
        &self.sample_rate_options
    }

    pub fn buffer_size_options(&self) -> &Option<BufferSizeOptions> {
        &self.buffer_size_options
    }

    pub fn audio_config(&self) -> Option<&AudioConfig> {
        self.audio_config.as_ref()
    }

    pub fn midi_config(&self) -> Option<&MidiConfig> {
        self.midi_config.as_ref()
    }

    pub fn audio_config_info(&self) -> &Option<AudioConfigInfo> {
        &self.audio_config_info
    }

    pub fn audio_server_unavailable(&self) -> bool {
        !self.os_info.audio_servers_info()[self.state.audio_server_index].available
    }

    pub fn midi_server_unavailable(&self) -> bool {
        !self.os_info.midi_servers_info()[self.state.midi_server_index].available
    }

    pub fn audio_device_not_selected(&self) -> bool {
        if let Some(AudioServerDevices::MultipleDevices(_)) =
            &self.os_info.audio_servers_info()[self.state.audio_server_index].devices
        {
            // First option is "None"
            return self.state.audio_device_index == 0;
        }

        false
    }

    pub fn audio_device_playback_only(&self) -> bool {
        if let Some(AudioServerDevices::MultipleDevices(devices)) =
            &self.os_info.audio_servers_info()[self.state.audio_server_index].devices
        {
            // First option is "None"
            if self.state.audio_device_index > 0 {
                return devices[self.state.audio_device_index - 1]
                    .in_ports
                    .is_empty();
            }
        }

        false
    }

    pub fn can_start(&self) -> bool {
        !self.audio_server_unavailable() && !self.audio_device_not_selected()
    }

    pub fn audio_in_port_options(&self) -> Option<&[String]> {
        if let Some(device) = self.current_audio_device_info() {
            if !device.in_ports.is_empty() {
                return Some(device.in_ports.as_slice());
            }
        }

        None
    }

    pub fn audio_out_port_options(&self) -> Option<&[String]> {
        if let Some(device) = self.current_audio_device_info() {
            if !device.out_ports.is_empty() {
                return Some(device.out_ports.as_slice());
            }
        }

        None
    }

    pub fn midi_in_port_options(&self) -> Option<&[String]> {
        if !self.midi_in_port_options.is_empty() {
            Some(self.midi_in_port_options.as_slice())
        } else {
            None
        }
    }

    pub fn midi_out_port_options(&self) -> Option<&[String]> {
        if !self.midi_out_port_options.is_empty() {
            Some(self.midi_out_port_options.as_slice())
        } else {
            None
        }
    }

    fn current_audio_device_info(&self) -> Option<&SystemDeviceInfo> {
        if let Some(devices) =
            &self.os_info.audio_servers_info()[self.state.audio_server_index].devices
        {
            match &devices {
                AudioServerDevices::SingleDevice(d) => Some(d),
                AudioServerDevices::MultipleDevices(d) => {
                    // The first device is "None"
                    if self.state.audio_device_index > 0 {
                        Some(&d[self.state.audio_device_index - 1])
                    } else {
                        None
                    }
                }
            }
        } else {
            None
        }
    }

    fn build_audio_config(&mut self) -> Option<AudioConfig> {
        let server_info = &self.os_info.audio_servers_info()[self.state.audio_server_index];

        if !server_info.available {
            return None;
        }

        if self.state.audio_out_busses.is_empty() {
            return None;
        }

        if let Some(device) = self.current_audio_device_info() {
            let sample_rate = if self.sample_rate_options.is_some() {
                if self.state.sample_rate_index == 0 {
                    // First selection is always "Auto"
                    None
                } else {
                    Some(device.sample_rates[self.state.sample_rate_index - 1])
                }
            } else {
                None
            };

            let buffer_size = if let Some(buffer_size_options) = &self.buffer_size_options() {
                match buffer_size_options {
                    BufferSizeOptions::Constant { .. } => None,
                    BufferSizeOptions::Range { .. } => Some(self.state.buffer_size),
                    BufferSizeOptions::UnknownSize => None,
                }
            } else {
                None
            };

            let in_busses = self
                .state
                .audio_in_busses
                .iter()
                .map(|d| AudioBusConfig {
                    id: d.id.clone(),
                    system_ports: d.system_ports.clone(),
                })
                .collect();
            let out_busses = self
                .state
                .audio_out_busses
                .iter()
                .map(|d| AudioBusConfig {
                    id: d.id.clone(),
                    system_ports: d.system_ports.clone(),
                })
                .collect();

            Some(AudioConfig {
                server: server_info.name.clone(),
                system_device: device.name.clone(),
                in_busses,
                out_busses,
                sample_rate,
                buffer_size,
            })
        } else {
            return None;
        }
    }

    fn build_midi_config(&mut self) -> Option<MidiConfig> {
        let server_info = &self.os_info.midi_servers_info()[self.state.midi_server_index];

        if !server_info.available {
            return None;
        }

        if self.state.midi_in_controllers.is_empty() && self.state.midi_out_controllers.is_empty() {
            return None;
        }

        let in_controllers = self
            .state
            .midi_in_controllers
            .iter()
            .map(|d| MidiControllerConfig {
                id: d.id.clone(),
                system_port: d.system_port.clone(),
            })
            .collect();
        let out_controllers = self
            .state
            .midi_out_controllers
            .iter()
            .map(|d| MidiControllerConfig {
                id: d.id.clone(),
                system_port: d.system_port.clone(),
            })
            .collect();

        Some(MidiConfig {
            server: server_info.name.clone(),
            in_controllers,
            out_controllers,
        })
    }
}
