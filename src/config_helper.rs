use crate::{AudioServerConfig, BufferSizeInfo, DevicesInfo, DuplexDeviceType};

pub static DEFAULT_BUFFER_SIZE: u32 = 512;

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

pub struct DeviceIOConfigState {
    pub audio_server: usize,
    pub duplex_device: usize,

    pub half_duplex_in_device: usize,
    pub half_duplex_out_device: usize,

    pub sample_rate_index: usize,

    pub buffer_size: u32,
}

impl Default for DeviceIOConfigState {
    fn default() -> Self {
        Self {
            audio_server: 0,
            duplex_device: 0,

            half_duplex_in_device: 0,
            half_duplex_out_device: 0,

            sample_rate_index: 0,

            buffer_size: DEFAULT_BUFFER_SIZE,
        }
    }
}

pub enum HalfDuplexSelection {
    NotRelevant,
    Relevant {
        half_duplex_in: RadioSelection,
        half_duplex_out: RadioSelection,
    },
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

pub struct DeviceIOConfigHelper {
    devices_info: DevicesInfo,

    audio_server_selection: RadioSelection,
    duplex_device_selection: RadioSelection,

    half_duplex_selection: HalfDuplexSelection,

    sample_rate_selection: RadioSelection,

    buffer_size_selection: BufferSizeSelection,

    audio_config: AudioServerConfig,

    estimated_latency: u32,
    estimated_latency_ms: f64,
    sample_rate: u32,
}

impl Default for DeviceIOConfigHelper {
    fn default() -> Self {
        let mut new_self = Self {
            devices_info: DevicesInfo::new(),
            audio_server_selection: RadioSelection::new(),
            duplex_device_selection: RadioSelection::new(),

            half_duplex_selection: HalfDuplexSelection::NotRelevant,

            sample_rate_selection: RadioSelection::new(),

            buffer_size_selection: BufferSizeSelection::UnknownSize,

            audio_config: AudioServerConfig::default(),

            estimated_latency: 1,
            estimated_latency_ms: 1.0,
            sample_rate: 1,
        };

        new_self.audio_server_selection.options = new_self
            .devices_info
            .audio_servers_info()
            .iter()
            .map(|s| s.name.clone())
            .collect();

        new_self.duplex_device_selection.options = new_self.devices_info.audio_servers_info()
            [new_self.audio_server_selection.selected]
            .devices
            .iter()
            .map(|d| d.name.clone())
            .collect();

        match &new_self.devices_info.audio_servers_info()[new_self.audio_server_selection.selected]
            .devices[new_self.duplex_device_selection.selected]
            .devices
        {
            DuplexDeviceType::Full { .. } => {
                new_self.half_duplex_selection = HalfDuplexSelection::NotRelevant;
            }
            DuplexDeviceType::Half {
                in_devices,
                out_devices,
            } => {
                let mut in_options: Vec<String> =
                    in_devices.iter().map(|d| d.name.clone()).collect();
                let mut out_options: Vec<String> =
                    out_devices.iter().map(|d| d.name.clone()).collect();

                if in_options.is_empty() {
                    in_options.push(String::from("Unavailable"));
                }
                if out_options.is_empty() {
                    out_options.push(String::from("Unavailable"));
                }

                new_self.half_duplex_selection = HalfDuplexSelection::Relevant {
                    half_duplex_in: RadioSelection {
                        selected: 0,
                        options: in_options,
                    },
                    half_duplex_out: RadioSelection {
                        selected: 0,
                        options: out_options,
                    },
                };
            }
        }

        new_self.sample_rate_selection.options = vec![String::from("Auto")]; // Always have "Auto" as the first option for sample rate.
        new_self.sample_rate_selection.options.append(
            &mut new_self.devices_info.audio_servers_info()
                [new_self.audio_server_selection.selected]
                .devices[new_self.duplex_device_selection.selected]
                .sample_rates
                .iter()
                .map(|s| format!("{}", s))
                .collect(),
        );

        match &new_self.devices_info.audio_servers_info()[new_self.audio_server_selection.selected]
            .devices[new_self.duplex_device_selection.selected]
            .buffer_size
        {
            BufferSizeInfo::ConstantSize(size) => {
                new_self.buffer_size_selection = BufferSizeSelection::Constant { auto_value: *size }
            }
            BufferSizeInfo::Range { min, max } => {
                new_self.buffer_size_selection = BufferSizeSelection::Range {
                    selected: DEFAULT_BUFFER_SIZE.min(*min).max(*max),
                    auto_value: DEFAULT_BUFFER_SIZE.min(*min).max(*max),
                    min: *min,
                    max: *max,
                };
            }
            BufferSizeInfo::UnknownSize => {
                new_self.buffer_size_selection = BufferSizeSelection::UnknownSize;
            }
        }

        new_self.build_audio_config();

        new_self.estimated_latency = new_self
            .devices_info
            .estimated_latency(&new_self.audio_config);
        new_self.sample_rate = new_self.devices_info.sample_rate(&new_self.audio_config);

        new_self.estimated_latency_ms =
            1_000.0 * f64::from(new_self.estimated_latency) / f64::from(new_self.sample_rate);

        new_self
    }
}

impl DeviceIOConfigHelper {
    pub fn update(&mut self, state: &mut DeviceIOConfigState) -> bool {
        let mut changed = false;

        let mut duplex_device_changed = false;

        if state.audio_server != self.audio_server_selection.selected {
            let index = state
                .audio_server
                .min(self.audio_server_selection.options.len() - 1);
            state.audio_server = index;
            self.audio_server_selection.selected = index;

            // Update duplex device
            self.duplex_device_selection.options = self.devices_info.audio_servers_info()
                [self.audio_server_selection.selected]
                .devices
                .iter()
                .map(|d| d.name.clone())
                .collect();
            self.duplex_device_selection.selected = 0;
            state.duplex_device = 0;

            duplex_device_changed = true;
            changed = true;
        }

        if state.duplex_device != self.duplex_device_selection.selected {
            let index = state
                .duplex_device
                .min(self.duplex_device_selection.options.len() - 1);
            state.duplex_device = index;
            self.duplex_device_selection.selected = index;

            duplex_device_changed = true;
            changed = true;
        }

        if duplex_device_changed {
            match &self.devices_info.audio_servers_info()[self.audio_server_selection.selected]
                .devices[self.duplex_device_selection.selected]
                .devices
            {
                DuplexDeviceType::Full { .. } => {
                    self.half_duplex_selection = HalfDuplexSelection::NotRelevant;
                }
                DuplexDeviceType::Half {
                    in_devices,
                    out_devices,
                } => {
                    let mut in_options: Vec<String> =
                        in_devices.iter().map(|d| d.name.clone()).collect();
                    let mut out_options: Vec<String> =
                        out_devices.iter().map(|d| d.name.clone()).collect();

                    if in_options.is_empty() {
                        in_options.push(String::from("Unavailable"));
                    }
                    if out_options.is_empty() {
                        out_options.push(String::from("Unavailable"));
                    }

                    self.half_duplex_selection = HalfDuplexSelection::Relevant {
                        half_duplex_in: RadioSelection {
                            selected: 0,
                            options: in_options,
                        },
                        half_duplex_out: RadioSelection {
                            selected: 0,
                            options: out_options,
                        },
                    };

                    state.half_duplex_in_device = 0;
                    state.half_duplex_out_device = 0;
                }
            }

            self.sample_rate_selection.options = vec![String::from("Auto")]; // Always have "Auto" as the first option for sample rate.
            self.sample_rate_selection.options.append(
                &mut self.devices_info.audio_servers_info()[self.audio_server_selection.selected]
                    .devices[self.duplex_device_selection.selected]
                    .sample_rates
                    .iter()
                    .map(|s| format!("{}", s))
                    .collect(),
            );
            self.sample_rate_selection.selected = 0;
            state.sample_rate_index = 0;

            state.buffer_size = match &self.devices_info.audio_servers_info()
                [self.audio_server_selection.selected]
                .devices[self.duplex_device_selection.selected]
                .buffer_size
            {
                BufferSizeInfo::ConstantSize(size) => {
                    self.buffer_size_selection =
                        BufferSizeSelection::Constant { auto_value: *size };

                    *size
                }
                BufferSizeInfo::Range { min, max } => {
                    self.buffer_size_selection = BufferSizeSelection::Range {
                        selected: DEFAULT_BUFFER_SIZE.min(*min).max(*max),
                        auto_value: DEFAULT_BUFFER_SIZE.min(*min).max(*max),
                        min: *min,
                        max: *max,
                    };

                    DEFAULT_BUFFER_SIZE.min(*min).max(*max)
                }
                BufferSizeInfo::UnknownSize => {
                    self.buffer_size_selection = BufferSizeSelection::UnknownSize;
                    DEFAULT_BUFFER_SIZE
                }
            };
        }

        match &mut self.half_duplex_selection {
            HalfDuplexSelection::NotRelevant => {}
            HalfDuplexSelection::Relevant {
                half_duplex_in,
                half_duplex_out,
            } => {
                if state.half_duplex_in_device != half_duplex_in.selected {
                    let index = state
                        .half_duplex_in_device
                        .min(half_duplex_in.options.len() - 1);
                    state.half_duplex_in_device = index;
                    half_duplex_in.selected = index;

                    changed = true;
                }
                if state.half_duplex_out_device != half_duplex_out.selected {
                    let index = state
                        .half_duplex_out_device
                        .min(half_duplex_out.options.len() - 1);
                    state.half_duplex_out_device = index;
                    half_duplex_out.selected = index;

                    changed = true;
                }
            }
        }

        if state.sample_rate_index != self.sample_rate_selection.selected {
            let index = state
                .sample_rate_index
                .min(self.sample_rate_selection.options.len() - 1);
            state.sample_rate_index = index;
            self.sample_rate_selection.selected = index;

            changed = true;
        }

        match &mut self.buffer_size_selection {
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

                    changed = true;
                }
            }
        }

        if changed {
            self.build_audio_config();

            self.estimated_latency = self.devices_info.estimated_latency(&self.audio_config);
            self.sample_rate = self.devices_info.sample_rate(&self.audio_config);
            self.estimated_latency_ms =
                1_000.0 * f64::from(self.estimated_latency) / f64::from(self.sample_rate);
        }

        changed
    }

    pub fn audio_server(&self) -> &RadioSelection {
        &self.audio_server_selection
    }

    pub fn duplex_device(&self) -> &RadioSelection {
        &self.duplex_device_selection
    }

    pub fn half_duplex_devices(&self) -> &HalfDuplexSelection {
        &self.half_duplex_selection
    }

    pub fn sample_rate(&self) -> &RadioSelection {
        &self.sample_rate_selection
    }

    pub fn buffer_size(&self) -> &BufferSizeSelection {
        &self.buffer_size_selection
    }

    pub fn refresh_audio_servers(&mut self) {}

    pub fn refresh_midi_servers(&mut self) {}

    pub fn audio_server_config(&self) -> &AudioServerConfig {
        &self.audio_config
    }

    pub fn estimated_latency(&self) -> u32 {
        self.estimated_latency
    }
    pub fn estimated_latency_ms(&self) -> f64 {
        self.estimated_latency_ms
    }

    pub fn current_sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn current_server_available(&self) -> bool {
        self.devices_info.audio_servers_info()[self.audio_server_selection.selected].available
    }

    fn build_audio_config(&mut self) {
        let server_info =
            &self.devices_info.audio_servers_info()[self.audio_server_selection.selected];
        let device_info = &server_info.devices[self.duplex_device_selection.selected];

        let (system_half_duplex_in_device, system_half_duplex_out_device) =
            match &self.half_duplex_selection {
                HalfDuplexSelection::NotRelevant => (None, None),
                HalfDuplexSelection::Relevant {
                    half_duplex_in,
                    half_duplex_out,
                } => {
                    let in_name = &half_duplex_in.options[half_duplex_in.selected];
                    let out_name = &half_duplex_out.options[half_duplex_out.selected];

                    (
                        if in_name == "Unavailable" {
                            None
                        } else {
                            Some(in_name.clone())
                        },
                        if out_name == "Unavailable" {
                            None
                        } else {
                            Some(out_name.clone())
                        },
                    )
                }
            };

        let sample_rate = if self.sample_rate_selection.selected == 0 {
            // First selection is always "Auto"
            None
        } else {
            Some(device_info.sample_rates[self.sample_rate_selection.selected - 1])
        };

        let buffer_size = match self.buffer_size_selection {
            BufferSizeSelection::Constant { .. } => None,
            BufferSizeSelection::Range { selected, .. } => Some(selected),
            BufferSizeSelection::UnknownSize => None,
        };

        self.audio_config = AudioServerConfig {
            server: server_info.name.clone(),
            system_duplex_device: device_info.name.clone(),
            system_half_duplex_in_device,
            system_half_duplex_out_device,
            create_in_devices: vec![],
            create_out_devices: vec![],
            sample_rate,
            buffer_size,
        }
    }
}
