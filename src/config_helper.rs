use crate::{AudioServerConfig, AudioServerDevices, BufferSizeInfo, DevicesInfo, SystemDeviceInfo};

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
    os_info: DevicesInfo,

    audio_server_options: RadioSelection,
    device_selection: Option<RadioSelection>,

    sample_rate_options: Option<RadioSelection>,
    buffer_size_options: Option<BufferSizeSelection>,

    audio_config: Option<AudioServerConfig>,
    audio_config_info: Option<AudioConfigInfo>,
}

impl DeviceIOConfigHelper {
    pub fn new() -> (Self, DeviceIOConfigState) {
        let mut new_self = Self {
            os_info: DevicesInfo::new(),
            audio_server_options: RadioSelection::new(),
            device_selection: None,

            sample_rate_options: None,
            buffer_size_options: None,

            audio_config: None,
            audio_config_info: None,
        };

        new_self.audio_server_options.options = new_self
            .os_info
            .audio_servers_info()
            .iter()
            .map(|s| s.name.clone())
            .collect();

        let mut config_state = DeviceIOConfigState::default();

        // Force update on all available options.
        new_self.audio_server_options.selected = 1;

        new_self.update(&mut config_state);

        (new_self, config_state)
    }

    pub fn update(&mut self, state: &mut DeviceIOConfigState) -> bool {
        let mut changed = false;
        let mut device_changed = false;

        // Check if audio server selection changed

        if state.audio_server != self.audio_server_options.selected {
            let index = state
                .audio_server
                .min(self.audio_server_options.options.len() - 1);
            state.audio_server = index;
            self.audio_server_options.selected = index;

            // Rebuild available devices

            self.device_selection = if let Some(AudioServerDevices::MultipleDevices(devices)) =
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
            state.audio_device = 0;

            device_changed = true;
            changed = true;
        }

        // Check if device selection changed

        if let Some(device_selection) = &mut self.device_selection {
            if state.audio_device != device_selection.selected {
                let index = state.audio_device.min(device_selection.options.len() - 1);
                state.audio_device = index;
                device_selection.selected = index;

                device_changed = true;
                changed = true;
            }
        }

        // Rebuild available options for device

        if device_changed {
            self.sample_rate_options = if let Some(device) = self.current_device() {
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

                Some(RadioSelection {
                    selected: 0,
                    options: sample_rate_options,
                })
            } else {
                state.sample_rate_index = 0;

                self.buffer_size_options = None;
                state.buffer_size = DEFAULT_BUFFER_SIZE;

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

                changed = true;
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

                        changed = true;
                    }
                }
            }
        }

        // Rebuild audio config if changed

        if changed {
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

        changed
    }

    pub fn audio_server_options(&self) -> &RadioSelection {
        &self.audio_server_options
    }

    pub fn audio_device_options(&self) -> &Option<RadioSelection> {
        &self.device_selection
    }

    pub fn sample_rate_options(&self) -> &Option<RadioSelection> {
        &self.sample_rate_options
    }

    pub fn buffer_size_options(&self) -> &Option<BufferSizeSelection> {
        &self.buffer_size_options
    }

    pub fn refresh_audio_servers(&mut self) {}

    pub fn refresh_midi_servers(&mut self) {}

    pub fn audio_server_config(&self) -> &Option<AudioServerConfig> {
        &self.audio_config
    }

    pub fn audio_config_info(&self) -> &Option<AudioConfigInfo> {
        &self.audio_config_info
    }

    pub fn audio_server_unavailable(&self) -> bool {
        !self.os_info.audio_servers_info()[self.audio_server_options.selected].available
    }

    pub fn audio_device_not_selected(&self) -> bool {
        if let Some(AudioServerDevices::MultipleDevices(_)) =
            &self.os_info.audio_servers_info()[self.audio_server_options.selected].devices
        {
            // First option is "None"
            return self.device_selection.as_ref().unwrap().selected == 0;
        }

        false
    }

    pub fn audio_device_playback_only(&self) -> bool {
        if let Some(AudioServerDevices::MultipleDevices(devices)) =
            &self.os_info.audio_servers_info()[self.audio_server_options.selected].devices
        {
            // First option is "None"
            if self.device_selection.as_ref().unwrap().selected > 0 {
                return devices[self.device_selection.as_ref().unwrap().selected - 1]
                    .in_ports
                    .is_empty();
            }
        }

        false
    }

    pub fn can_start(&self) -> bool {
        !self.audio_server_unavailable() && !self.audio_device_not_selected()
    }

    fn current_device(&self) -> Option<&SystemDeviceInfo> {
        if let Some(devices) =
            &self.os_info.audio_servers_info()[self.audio_server_options.selected].devices
        {
            match &devices {
                AudioServerDevices::SingleDevice(d) => Some(d),
                AudioServerDevices::MultipleDevices(d) => {
                    // The first device is "None"
                    if self.device_selection.as_ref().unwrap().selected > 0 {
                        Some(&d[self.device_selection.as_ref().unwrap().selected - 1])
                    } else {
                        None
                    }
                }
            }
        } else {
            None
        }
    }

    fn build_audio_config(&mut self) -> Option<AudioServerConfig> {
        let server_info = &self.os_info.audio_servers_info()[self.audio_server_options.selected];

        if !server_info.available {
            return None;
        }

        if let Some(device) = self.current_device() {
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

            Some(AudioServerConfig {
                server: server_info.name.clone(),
                system_device: device.name.clone(),
                create_in_devices: vec![],
                create_out_devices: vec![],
                sample_rate,
                buffer_size,
            })
        } else {
            return None;
        }
    }
}
