use crate::{AudioServerConfig, BufferSizeInfo, DevicesInfo};

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
    pub audio_device: usize,

    pub sample_rate_index: usize,

    pub buffer_size: u32,
}

impl Default for DeviceIOConfigState {
    fn default() -> Self {
        Self {
            audio_server: 0,
            audio_device: 0,

            sample_rate_index: 0,

            buffer_size: DEFAULT_BUFFER_SIZE,
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
    devices_info: DevicesInfo,

    audio_server_selection: RadioSelection,
    device_selection: RadioSelection,

    sample_rate_selection: RadioSelection,
    buffer_size_selection: BufferSizeSelection,

    audio_config: Option<AudioServerConfig>,
    audio_config_info: Option<AudioConfigInfo>,
}

impl Default for DeviceIOConfigHelper {
    fn default() -> Self {
        let mut new_self = Self {
            devices_info: DevicesInfo::new(),
            audio_server_selection: RadioSelection::new(),
            device_selection: RadioSelection::new(),

            sample_rate_selection: RadioSelection::new(),
            buffer_size_selection: BufferSizeSelection::UnknownSize,

            audio_config: None,
            audio_config_info: None,
        };

        new_self.audio_server_selection.options = new_self
            .devices_info
            .audio_servers_info()
            .iter()
            .map(|s| s.name.clone())
            .collect();

        new_self.device_selection.options = vec![String::from("None")]; // Always have "None" as the first option.
        new_self.device_selection.options.append(
            &mut new_self.devices_info.audio_servers_info()
                [new_self.audio_server_selection.selected]
                .devices
                .iter()
                .map(|d| d.name.clone())
                .collect(),
        );

        new_self.sample_rate_selection.options = vec![String::from("Auto")]; // Always have "Auto" as the first option for sample rate.
        if new_self.device_selection.selected > 0 {
            new_self.sample_rate_selection.options.append(
                &mut new_self.devices_info.audio_servers_info()
                    [new_self.audio_server_selection.selected]
                    .devices[new_self.device_selection.selected - 1]
                    .sample_rates
                    .iter()
                    .map(|s| format!("{}", s))
                    .collect(),
            );

            match &new_self.devices_info.audio_servers_info()
                [new_self.audio_server_selection.selected]
                .devices[new_self.device_selection.selected]
                .buffer_size
            {
                BufferSizeInfo::ConstantSize(size) => {
                    new_self.buffer_size_selection =
                        BufferSizeSelection::Constant { auto_value: *size }
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
        }

        new_self.audio_config = new_self.build_audio_config();
        if let Some(audio_config) = &new_self.audio_config {
            let estimated_latency = new_self.devices_info.estimated_latency(audio_config);
            let sample_rate = new_self.devices_info.sample_rate(audio_config);

            new_self.audio_config_info = Some(AudioConfigInfo {
                estimated_latency,
                sample_rate,
                estimated_latency_ms: 1_000.0 * f64::from(estimated_latency)
                    / f64::from(sample_rate),
            })
        }

        new_self
    }
}

impl DeviceIOConfigHelper {
    pub fn update(&mut self, state: &mut DeviceIOConfigState) -> bool {
        let mut changed = false;

        let mut device_changed = false;

        if state.audio_server != self.audio_server_selection.selected {
            let index = state
                .audio_server
                .min(self.audio_server_selection.options.len() - 1);
            state.audio_server = index;
            self.audio_server_selection.selected = index;

            // Update device
            self.device_selection.options = vec![String::from("None")]; // Always have "None" as the first option.
            self.device_selection.options.append(
                &mut self.devices_info.audio_servers_info()[self.audio_server_selection.selected]
                    .devices
                    .iter()
                    .map(|d| d.name.clone())
                    .collect(),
            );
            self.device_selection.selected = 0;
            state.audio_device = 0;

            device_changed = true;
            changed = true;
        }

        if state.audio_device != self.device_selection.selected {
            let index = state
                .audio_device
                .min(self.device_selection.options.len() - 1);
            state.audio_device = index;
            self.device_selection.selected = index;

            device_changed = true;
            changed = true;
        }

        if device_changed {
            self.sample_rate_selection.options = vec![String::from("Auto")]; // Always have "Auto" as the first option for sample rate.
            if self.device_selection.selected > 0 {
                self.sample_rate_selection.options.append(
                    &mut self.devices_info.audio_servers_info()
                        [self.audio_server_selection.selected]
                        .devices[self.device_selection.selected - 1]
                        .sample_rates
                        .iter()
                        .map(|s| format!("{}", s))
                        .collect(),
                );
                self.sample_rate_selection.selected = 0;
                state.sample_rate_index = 0;

                state.buffer_size = match &self.devices_info.audio_servers_info()
                    [self.audio_server_selection.selected]
                    .devices[self.device_selection.selected - 1]
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
            } else {
                self.buffer_size_selection = BufferSizeSelection::UnknownSize;
                state.buffer_size = DEFAULT_BUFFER_SIZE;
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
            self.audio_config = self.build_audio_config();

            if let Some(audio_config) = &self.audio_config {
                let estimated_latency = self.devices_info.estimated_latency(audio_config);
                let sample_rate = self.devices_info.sample_rate(audio_config);

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

        changed
    }

    pub fn audio_server(&self) -> &RadioSelection {
        &self.audio_server_selection
    }

    pub fn audio_device(&self) -> &RadioSelection {
        &self.device_selection
    }

    pub fn sample_rate(&self) -> &RadioSelection {
        &self.sample_rate_selection
    }

    pub fn buffer_size(&self) -> &BufferSizeSelection {
        &self.buffer_size_selection
    }

    pub fn refresh_audio_servers(&mut self) {}

    pub fn refresh_midi_servers(&mut self) {}

    pub fn audio_server_config(&self) -> &Option<AudioServerConfig> {
        &self.audio_config
    }

    pub fn audio_config_info(&self) -> &Option<AudioConfigInfo> {
        &self.audio_config_info
    }

    pub fn current_server_available(&self) -> bool {
        self.devices_info.audio_servers_info()[self.audio_server_selection.selected].available
    }

    pub fn audio_device_selected(&self) -> bool {
        // First option is "None"
        self.device_selection.selected != 0
    }

    pub fn audio_device_playback_only(&self) -> bool {
        // First option is "None"
        if self.device_selection.selected > 0 {
            return self.devices_info.audio_servers_info()[self.audio_server_selection.selected]
                .devices[self.device_selection.selected - 1]
                .in_ports
                .is_empty();
        }
        false
    }

    pub fn can_start(&self) -> bool {
        self.current_server_available() && self.audio_device_selected()
    }

    fn build_audio_config(&mut self) -> Option<AudioServerConfig> {
        let server_info =
            &self.devices_info.audio_servers_info()[self.audio_server_selection.selected];

        if !server_info.available {
            return None;
        }

        // First device is "None"
        if self.device_selection.selected == 0 {
            return None;
        }

        let device_info = &server_info.devices[self.device_selection.selected - 1];

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

        Some(AudioServerConfig {
            server: server_info.name.clone(),
            system_device: device_info.name.clone(),
            create_in_devices: vec![],
            create_out_devices: vec![],
            sample_rate,
            buffer_size,
        })
    }
}
