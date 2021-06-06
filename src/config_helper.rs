use crate::{
    AudioBusConfig, AudioConfig, AudioServerDevices, BufferSizeInfo, DevicesInfo, MidiConfig,
    MidiControllerConfig, SystemDeviceInfo,
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
pub struct DeviceIOConfigState {
    pub audio_server: usize,
    pub audio_server_device: usize,

    pub midi_server: usize,

    pub sample_rate_index: usize,
    pub buffer_size: u32,

    pub audio_in_busses: Vec<AudioBusConfigState>,
    pub audio_out_busses: Vec<AudioBusConfigState>,

    pub midi_in_controllers: Vec<MidiControllerConfigState>,
    pub midi_out_controllers: Vec<MidiControllerConfigState>,
}

impl Default for DeviceIOConfigState {
    fn default() -> Self {
        Self {
            audio_server: 0,
            audio_server_device: 0,

            midi_server: 0,

            sample_rate_index: 0,
            buffer_size: DEFAULT_BUFFER_SIZE,

            audio_in_busses: Vec::new(),
            audio_out_busses: Vec::new(),

            midi_in_controllers: Vec::new(),
            midi_out_controllers: Vec::new(),
        }
    }
}

pub struct RadioSelection {
    pub selected: usize,
    pub options: Vec<String>,
}

impl RadioSelection {
    pub(crate) fn new() -> Self {
        Self {
            selected: 0,
            options: Vec::new(),
        }
    }
}

pub enum BufferSizeSelection {
    UnknownSize,
    Constant {
        auto_value: u32,
    },
    Range {
        selected: u32,
        auto_value: u32,
        min: u32,
        max: u32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AudioConfigInfo {
    pub estimated_latency: u32,
    pub estimated_latency_ms: f64,
    pub sample_rate: u32,
}

pub struct DeviceIOConfigHelper {
    os_info: DevicesInfo,

    audio_server_options: RadioSelection,
    audio_device_options: Option<RadioSelection>,

    midi_server_options: RadioSelection,

    sample_rate_options: Option<RadioSelection>,
    buffer_size_options: Option<BufferSizeSelection>,

    audio_config: Option<AudioConfig>,
    audio_config_info: Option<AudioConfigInfo>,

    midi_config: Option<MidiConfig>,

    audio_in_busses: Vec<AudioBusConfigState>,
    audio_out_busses: Vec<AudioBusConfigState>,

    midi_in_controllers: Vec<MidiControllerConfigState>,
    midi_out_controllers: Vec<MidiControllerConfigState>,

    midi_in_port_options: Vec<String>,
    midi_out_port_options: Vec<String>,

    created_audio_in_busses: usize,
    created_audio_out_busses: usize,
    created_midi_in_controllers: usize,
    created_midi_out_controllers: usize,
}

impl DeviceIOConfigHelper {
    pub fn new() -> (Self, DeviceIOConfigState) {
        let mut new_self = Self {
            os_info: DevicesInfo::new(),

            audio_server_options: RadioSelection::new(),
            audio_device_options: None,

            midi_server_options: RadioSelection::new(),

            sample_rate_options: None,
            buffer_size_options: None,

            audio_config: None,
            audio_config_info: None,

            midi_config: None,

            audio_in_busses: Vec::new(),
            audio_out_busses: Vec::new(),

            midi_in_controllers: Vec::new(),
            midi_out_controllers: Vec::new(),

            midi_in_port_options: Vec::new(),
            midi_out_port_options: Vec::new(),

            created_audio_in_busses: 1,
            created_audio_out_busses: 1,
            created_midi_in_controllers: 1,
            created_midi_out_controllers: 0,
        };

        new_self.audio_server_options.options = new_self
            .os_info
            .audio_servers_info()
            .iter()
            .map(|s| s.name.clone())
            .collect();

        new_self.midi_server_options.options = new_self
            .os_info
            .midi_servers_info()
            .iter()
            .map(|s| s.name.clone())
            .collect();

        let mut config_state = DeviceIOConfigState::default();

        // Force update on all available options.
        new_self.audio_server_options.selected = 1;
        new_self.midi_server_options.selected = 1;

        new_self.update(&mut config_state);

        (new_self, config_state)
    }

    pub fn update(&mut self, state: &mut DeviceIOConfigState) -> bool {
        let mut audio_changed = false;
        let mut audio_device_changed = false;

        // Check if audio server selection changed

        if state.audio_server != self.audio_server_options.selected {
            let index = state
                .audio_server
                .min(self.audio_server_options.options.len() - 1);
            state.audio_server = index;
            self.audio_server_options.selected = index;

            // Rebuild available devices

            self.audio_device_options = if let Some(AudioServerDevices::MultipleDevices(devices)) =
                &self.os_info.audio_servers_info()[self.audio_server_options.selected].devices
            {
                let mut options = vec![String::from("None")]; // Always have "None" as the first option.
                options.append(&mut devices.iter().map(|d| d.name.clone()).collect());

                Some(RadioSelection {
                    selected: 0,
                    options,
                })
            } else {
                None
            };
            state.audio_server_device = 0;

            audio_device_changed = true;
            audio_changed = true;
        }

        // Check if device selection changed

        if let Some(audio_device_options) = &mut self.audio_device_options {
            if state.audio_server_device != audio_device_options.selected {
                let index = state
                    .audio_server_device
                    .min(audio_device_options.options.len() - 1);
                state.audio_server_device = index;
                audio_device_options.selected = index;

                audio_device_changed = true;
                audio_changed = true;
            }
        }

        // Rebuild available options for device

        if audio_device_changed {
            self.sample_rate_options = if let Some(device) = self.current_audio_device_info() {
                // Sample rate options
                let mut sample_rate_options = vec![String::from("Auto")]; // Always have "Auto" as the first option for sample rate.
                sample_rate_options.append(
                    &mut device
                        .sample_rates
                        .iter()
                        .map(|s| format!("{}", s))
                        .collect(),
                );

                match &device.buffer_size {
                    BufferSizeInfo::ConstantSize(size) => {
                        self.buffer_size_options =
                            Some(BufferSizeSelection::Constant { auto_value: *size })
                    }
                    BufferSizeInfo::Range { min, max } => {
                        self.buffer_size_options = Some(BufferSizeSelection::Range {
                            selected: DEFAULT_BUFFER_SIZE.min(*min).max(*max),
                            auto_value: DEFAULT_BUFFER_SIZE.min(*min).max(*max),
                            min: *min,
                            max: *max,
                        });
                    }
                    BufferSizeInfo::UnknownSize => {
                        self.buffer_size_options = Some(BufferSizeSelection::UnknownSize);
                    }
                }

                // Make sure there is always at-least one output.
                let (audio_in_busses, audio_out_busses) = self.default_audio_busses();
                self.audio_in_busses = audio_in_busses.clone();
                self.audio_out_busses = audio_out_busses.clone();
                state.audio_in_busses = audio_in_busses;
                state.audio_out_busses = audio_out_busses;

                Some(RadioSelection {
                    selected: 0,
                    options: sample_rate_options,
                })
            } else {
                state.sample_rate_index = 0;

                self.buffer_size_options = None;
                state.buffer_size = DEFAULT_BUFFER_SIZE;

                self.audio_in_busses.clear();
                self.audio_out_busses.clear();
                state.audio_in_busses.clear();
                state.audio_out_busses.clear();

                None
            }
        }

        // Check if sample rate selection changed

        if let Some(sample_rate_options) = &mut self.sample_rate_options {
            if state.sample_rate_index != sample_rate_options.selected {
                let index = state
                    .sample_rate_index
                    .min(sample_rate_options.options.len() - 1);
                state.sample_rate_index = index;
                sample_rate_options.selected = index;

                audio_changed = true;
            }
        }

        // Check if buffer size selection changed

        if let Some(buffer_size_options) = &mut self.buffer_size_options {
            match buffer_size_options {
                BufferSizeSelection::UnknownSize => {
                    state.buffer_size = DEFAULT_BUFFER_SIZE; // Not that necessary, but make sure the user doesn't try to change this.
                }
                BufferSizeSelection::Constant { auto_value } => {
                    state.buffer_size = *auto_value; // Make sure the user doesn't try to change this.
                }
                BufferSizeSelection::Range {
                    selected, min, max, ..
                } => {
                    if state.buffer_size != *selected {
                        let size = state.buffer_size.min(*min).max(*max);
                        state.buffer_size = size;
                        *selected = size;

                        audio_changed = true;
                    }
                }
            }
        }

        // Check if audio busses have changed

        // Flippin' immutable and mutable borrows. There is probably a way to elegantly use the `current_audio_device_info()` function,
        // but I don't feel like messing with it right now. I'm inlining the function here instead.
        let device = if let Some(devices) =
            &self.os_info.audio_servers_info()[self.audio_server_options.selected].devices
        {
            match &devices {
                AudioServerDevices::SingleDevice(d) => Some(d),
                AudioServerDevices::MultipleDevices(d) => {
                    // The first device is "None"
                    if self.audio_device_options.as_ref().unwrap().selected > 0 {
                        Some(&d[self.audio_device_options.as_ref().unwrap().selected - 1])
                    } else {
                        None
                    }
                }
            }
        } else {
            None
        };
        if let Some(device) = device {
            if self.audio_in_busses != state.audio_in_busses
                || self.audio_out_busses != state.audio_out_busses
            {
                self.audio_in_busses.clear();
                self.audio_out_busses.clear();

                // Delete any busses marked for deletion.
                state.audio_in_busses.retain(|d| !d.do_delete);
                state.audio_out_busses.retain(|d| !d.do_delete);

                for bus in state.audio_in_busses.iter_mut() {
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
                        self.audio_in_busses.push(bus.clone());
                    }
                }

                for bus in state.audio_out_busses.iter_mut() {
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
                        self.audio_out_busses.push(bus.clone());
                    }
                }

                // Make sure there is always at-least one output bus.
                if state.audio_out_busses.is_empty() {
                    let (_, audio_out_busses) = self.default_audio_busses();
                    self.audio_out_busses = audio_out_busses;
                }

                // Make sure state matches.
                state.audio_in_busses = self.audio_in_busses.clone();
                state.audio_out_busses = self.audio_out_busses.clone();

                audio_changed = true;
            }
        }

        // Rebuild audio config if changed

        if audio_changed {
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

        if state.midi_server != self.midi_server_options.selected {
            let index = state
                .midi_server
                .min(self.midi_server_options.options.len() - 1);
            state.midi_server = index;
            self.midi_server_options.selected = index;

            self.midi_in_port_options = self.os_info.midi_servers_info()
                [self.midi_server_options.selected]
                .in_devices
                .iter()
                .map(|d| d.name.clone())
                .collect();
            self.midi_out_port_options = self.os_info.midi_servers_info()
                [self.midi_server_options.selected]
                .out_devices
                .iter()
                .map(|d| d.name.clone())
                .collect();

            let (midi_in_controllers, midi_out_controllers) = self.default_midi_controllers();
            self.midi_in_controllers = midi_in_controllers.clone();
            self.midi_out_controllers = midi_out_controllers.clone();
            state.midi_in_controllers = midi_in_controllers;
            state.midi_out_controllers = midi_out_controllers;

            midi_changed = true;
        }

        // Check if midi controllers have changed

        if self.midi_in_controllers != state.midi_in_controllers
            || self.midi_out_controllers != state.midi_out_controllers
        {
            self.midi_in_controllers.clear();
            self.midi_out_controllers.clear();

            // Delete any controllers marked for deletion.
            state.midi_in_controllers.retain(|d| !d.do_delete);
            state.midi_out_controllers.retain(|d| !d.do_delete);

            for controller in state.midi_in_controllers.iter_mut() {
                let mut device_exists = false;
                for device in self.os_info.midi_servers_info()[self.midi_server_options.selected]
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
                    self.midi_in_controllers.push(controller.clone());
                }
            }

            for controller in state.midi_out_controllers.iter_mut() {
                let mut device_exists = false;
                for device in self.os_info.midi_servers_info()[self.midi_server_options.selected]
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
                    self.midi_out_controllers.push(controller.clone());
                }
            }

            // Make sure state matches.
            state.midi_in_controllers = self.midi_in_controllers.clone();
            state.midi_out_controllers = self.midi_out_controllers.clone();

            midi_changed = true;
        }

        // Rebuild midiconfig if changed

        if midi_changed {
            self.midi_config = self.build_midi_config();
        }

        audio_changed || midi_changed
    }

    pub fn new_audio_in_bus(&mut self) -> Option<AudioBusConfigState> {
        let new_bus = if let Some(device) = self.current_audio_device_info() {
            if device.in_ports.len() == 0 {
                None
            } else {
                // Find the index of the next available port.
                let mut next_port_i = 1;
                if let Some(last_bus) = self.audio_in_busses.last() {
                    if let Some(last_port) = last_bus.system_ports.last() {
                        for port in device.in_ports.iter() {
                            if port == last_port {
                                break;
                            }
                            next_port_i += 1;
                        }
                    }
                }
                if next_port_i >= device.in_ports.len() {
                    next_port_i = 0;
                }

                Some(AudioBusConfigState {
                    id: format!("Mic #{}", self.created_audio_in_busses + 1),
                    system_ports: vec![device.in_ports[next_port_i].clone()],
                    do_delete: false,
                })
            }
        } else {
            None
        };

        if new_bus.is_some() {
            self.created_audio_in_busses += 1;
        }

        new_bus
    }

    pub fn new_audio_out_bus(&mut self) -> Option<AudioBusConfigState> {
        let next_port = |device: &SystemDeviceInfo| {
            // Find the index of the next available port.
            let mut next_port_i = 1;
            if let Some(last_bus) = self.audio_out_busses.last() {
                if let Some(last_port) = last_bus.system_ports.last() {
                    for port in device.out_ports.iter() {
                        if port == last_port {
                            break;
                        }
                        next_port_i += 1;
                    }
                }
            }
            if next_port_i >= device.out_ports.len() {
                next_port_i = 0;
            }
            next_port_i
        };

        let new_bus = if let Some(device) = self.current_audio_device_info() {
            if device.out_ports.len() == 0 {
                None
            } else if device.out_ports.len() == 1 {
                let next_port_i = next_port(device);

                Some(AudioBusConfigState {
                    id: format!("Mono Speaker #{}", self.created_audio_out_busses + 1),
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

                Some(AudioBusConfigState {
                    id: format!("Stereo Speakers #{}", self.created_audio_out_busses + 1),
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
            self.created_audio_out_busses += 1;
        }

        new_bus
    }

    pub fn new_midi_in_controller(&mut self) -> Option<MidiControllerConfigState> {
        let new_controller = if !self.midi_in_port_options.is_empty() {
            // Find the index of the next available port.
            let mut next_port_i = 1;
            if let Some(last_controller) = self.midi_in_controllers.last() {
                for port in self.midi_in_port_options.iter() {
                    if port == &last_controller.system_port {
                        break;
                    }
                    next_port_i += 1;
                }
            }
            if next_port_i >= self.midi_in_port_options.len() {
                next_port_i = 0;
            }

            Some(MidiControllerConfigState {
                id: format!(
                    "MIDI In Controller #{}",
                    self.created_midi_in_controllers + 1
                ),
                system_port: self.midi_in_port_options[next_port_i].clone(),
                do_delete: false,
            })
        } else {
            None
        };

        if new_controller.is_some() {
            self.created_midi_in_controllers += 1;
        }

        new_controller
    }

    pub fn new_midi_out_controller(&mut self) -> Option<MidiControllerConfigState> {
        let new_controller = if !self.midi_out_port_options.is_empty() {
            // Find the index of the next available port.
            let mut next_port_i = 1;
            if let Some(last_controller) = self.midi_out_controllers.last() {
                for port in self.midi_out_port_options.iter() {
                    if port == &last_controller.system_port {
                        break;
                    }
                    next_port_i += 1;
                }
            }
            if next_port_i >= self.midi_out_port_options.len() {
                next_port_i = 0;
            }

            Some(MidiControllerConfigState {
                id: format!(
                    "MIDI Out Controller #{}",
                    self.created_midi_out_controllers + 1,
                ),
                system_port: self.midi_out_port_options[next_port_i].clone(),
                do_delete: false,
            })
        } else {
            None
        };

        if new_controller.is_some() {
            self.created_midi_out_controllers += 1;
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

        if let Some(device) = self.os_info.midi_servers_info()[self.midi_server_options.selected]
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

    pub fn audio_server_options(&self) -> &RadioSelection {
        &self.audio_server_options
    }

    pub fn audio_server_device_options(&self) -> &Option<RadioSelection> {
        &self.audio_device_options
    }

    pub fn midi_server_options(&self) -> &RadioSelection {
        &self.midi_server_options
    }

    pub fn sample_rate_options(&self) -> &Option<RadioSelection> {
        &self.sample_rate_options
    }

    pub fn buffer_size_options(&self) -> &Option<BufferSizeSelection> {
        &self.buffer_size_options
    }

    pub fn refresh_audio_servers(&mut self, state: &mut DeviceIOConfigState) {
        let mut prev_state = state.clone();

        self.os_info.refresh_audio_servers();

        self.audio_server_options.options = self
            .os_info
            .audio_servers_info()
            .iter()
            .map(|s| s.name.clone())
            .collect();

        // Force update on all available audio options.
        self.audio_server_options.selected = state.audio_server + 1;
        self.update(state);

        // Try to keep settings of previous state.
        self.update(&mut prev_state);

        *state = prev_state;
    }

    pub fn refresh_midi_servers(&mut self, state: &mut DeviceIOConfigState) {
        let mut prev_state = state.clone();

        self.os_info.refresh_midi_servers();

        self.midi_server_options.options = self
            .os_info
            .midi_servers_info()
            .iter()
            .map(|s| s.name.clone())
            .collect();

        // Force update on all available midi options.
        self.midi_server_options.selected = state.midi_server + 1;
        self.update(state);

        // Try to keep settings of previous state.
        self.update(&mut prev_state);
        *state = prev_state;
    }

    pub fn audio_server_config(&self) -> &Option<AudioConfig> {
        &self.audio_config
    }

    pub fn audio_config_info(&self) -> &Option<AudioConfigInfo> {
        &self.audio_config_info
    }

    pub fn audio_server_unavailable(&self) -> bool {
        !self.os_info.audio_servers_info()[self.audio_server_options.selected].available
    }

    pub fn midi_server_unavailable(&self) -> bool {
        !self.os_info.midi_servers_info()[self.midi_server_options.selected].available
    }

    pub fn audio_server_device_not_selected(&self) -> bool {
        if let Some(AudioServerDevices::MultipleDevices(_)) =
            &self.os_info.audio_servers_info()[self.audio_server_options.selected].devices
        {
            // First option is "None"
            return self.audio_device_options.as_ref().unwrap().selected == 0;
        }

        false
    }

    pub fn audio_server_device_playback_only(&self) -> bool {
        if let Some(AudioServerDevices::MultipleDevices(devices)) =
            &self.os_info.audio_servers_info()[self.audio_server_options.selected].devices
        {
            // First option is "None"
            if self.audio_device_options.as_ref().unwrap().selected > 0 {
                return devices[self.audio_device_options.as_ref().unwrap().selected - 1]
                    .in_ports
                    .is_empty();
            }
        }

        false
    }

    pub fn can_start(&self) -> bool {
        !self.audio_server_unavailable() && !self.audio_server_device_not_selected()
    }

    pub fn audio_in_bus_config(&self) -> Option<(&[AudioBusConfigState], &[String])> {
        if let Some(device) = self.current_audio_device_info() {
            if !device.in_ports.is_empty() {
                return Some((self.audio_in_busses.as_slice(), device.in_ports.as_slice()));
            }
        }

        None
    }

    pub fn audio_out_bus_config(&self) -> Option<(&[AudioBusConfigState], &[String])> {
        if let Some(device) = self.current_audio_device_info() {
            if !device.out_ports.is_empty() {
                return Some((
                    self.audio_out_busses.as_slice(),
                    device.out_ports.as_slice(),
                ));
            }
        }

        None
    }

    pub fn midi_in_controller_config(&self) -> Option<(&[MidiControllerConfigState], &[String])> {
        if !self.midi_in_port_options.is_empty() {
            Some((
                self.midi_in_controllers.as_slice(),
                self.midi_in_port_options.as_slice(),
            ))
        } else {
            None
        }
    }

    pub fn midi_out_controller_config(&self) -> Option<(&[MidiControllerConfigState], &[String])> {
        if !self.midi_out_port_options.is_empty() {
            Some((
                self.midi_out_controllers.as_slice(),
                self.midi_out_port_options.as_slice(),
            ))
        } else {
            None
        }
    }

    fn current_audio_device_info(&self) -> Option<&SystemDeviceInfo> {
        if let Some(devices) =
            &self.os_info.audio_servers_info()[self.audio_server_options.selected].devices
        {
            match &devices {
                AudioServerDevices::SingleDevice(d) => Some(d),
                AudioServerDevices::MultipleDevices(d) => {
                    // The first device is "None"
                    if self.audio_device_options.as_ref().unwrap().selected > 0 {
                        Some(&d[self.audio_device_options.as_ref().unwrap().selected - 1])
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
        let server_info = &self.os_info.audio_servers_info()[self.audio_server_options.selected];

        if !server_info.available {
            return None;
        }

        if let Some(device) = self.current_audio_device_info() {
            let sample_rate = if let Some(sample_rate_options) = &self.sample_rate_options() {
                if sample_rate_options.selected == 0 {
                    // First selection is always "Auto"
                    None
                } else {
                    Some(device.sample_rates[sample_rate_options.selected - 1])
                }
            } else {
                None
            };

            let buffer_size = if let Some(buffer_size_options) = &self.buffer_size_options() {
                match buffer_size_options {
                    BufferSizeSelection::Constant { .. } => None,
                    BufferSizeSelection::Range { selected, .. } => Some(*selected),
                    BufferSizeSelection::UnknownSize => None,
                }
            } else {
                None
            };

            let in_busses = self
                .audio_in_busses
                .iter()
                .map(|d| AudioBusConfig {
                    id: d.id.clone(),
                    system_ports: d.system_ports.clone(),
                })
                .collect();
            let out_busses = self
                .audio_in_busses
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
        let server_info = &self.os_info.midi_servers_info()[self.midi_server_options.selected];

        if !server_info.available {
            return None;
        }

        if self.midi_in_controllers.is_empty() && self.midi_out_controllers.is_empty() {
            return None;
        }

        let in_controllers = self
            .midi_in_controllers
            .iter()
            .map(|d| MidiControllerConfig {
                id: d.id.clone(),
                system_port: d.system_port.clone(),
            })
            .collect();
        let out_controllers = self
            .midi_in_controllers
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
