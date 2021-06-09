#[derive(Debug, Clone, Default)]
pub struct DisplayState {
    audio_server_options: Vec<String>,
    selected_audio_server: usize,

    midi_server_options: Vec<String>,
    selected_midi_server: usize,

    audio_device_options: Vec<String>,
    selected_audio_device: usize,

    sample_rate_options: Vec<u32>,
    selected_sample_rate_index: usize,

    buffer_size_range: BufferSizeRange,
    selected_buffer_size: u32,

    audio_in_port_options: Vec<String>,
    audio_out_port_options: Vec<String>,

    midi_in_port_options: Vec<String>,
    midi_out_port_options: Vec<String>,

    audio_in_busses: Vec<AudioBus>,
    audio_out_busses: Vec<AudioBus>,

    midi_in_controllers: Vec<MidiController>,
    midi_out_controllers: Vec<MidiController>,

    is_valid: bool,
}

#[derive(Debug, Clone)]
pub struct AudioBus {
    id: String,
    system_ports: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct MidiController {
    id: String,
    system_port: String,
}

#[derive(Debug, Copy, Clone, Default)]
pub struct BufferSizeRange {
    pub min: u32,
    pub max: u32,
}

#[derive(Debug, Clone)]
pub struct SystemOptions {
    audio_server_info: Vec<AudioServerInfo>,
    midi_server_info: Vec<MidiServerInfo>,

    display_state: DisplayState,

    // For loading the default config
    default_audio_server: usize,
    default_midi_server: usize,
}

impl SystemOptions {
    pub(crate) fn new(
        audio_server_info: Vec<AudioServerInfo>,
        midi_server_info: Vec<MidiServerInfo>,

        default_audio_server: usize,
        default_midi_server: usize,
    ) -> Self {
        let mut new_self = Self {
            audio_server_info,
            midi_server_info,

            display_state: DisplayState::default(),

            default_audio_server,
            default_midi_server,
        };

        new_self.display_state.audio_server_options =
            audio_server_info.iter().map(|s| s.name.clone()).collect();
        new_self.display_state.midi_server_options =
            midi_server_info.iter().map(|s| s.name.clone()).collect();

        new_self.set_audio_defaults();
        new_self.set_midi_defaults();

        new_self
    }

    pub fn select_audio_server(&mut self, index: usize) {
        let index = index.min(self.display_state.audio_server_options.len() - 1);
        if self.display_state.selected_audio_server != index {
            self.display_state.selected_audio_server = index;

            self.display_state.audio_device_options = self.audio_server_info
                [self.display_state.selected_audio_server]
                .devices
                .iter()
                .map(|d| d.name.clone())
                .collect();

            self.set_defaults_for_current_audio_server();
        }
    }

    pub fn select_audio_device(&mut self, index: usize) {
        if self.display_state.audio_device_options.len() > 0 {
            let index = index.min(self.display_state.audio_device_options.len() - 1);
            if self.display_state.selected_audio_device != index {
                self.display_state.selected_audio_device = index;

                self.set_defaults_for_current_audio_device();
            }
        } else {
            self.display_state.is_valid = false;
        }
    }

    pub fn select_sample_rate(&mut self, index: usize) {
        if self.display_state.sample_rate_options.len() > 0 {
            let index = index.min(self.display_state.sample_rate_options.len() - 1);
            if self.display_state.selected_sample_rate_index != index {
                self.display_state.selected_sample_rate_index = index;
            }
        } else {
            self.display_state.is_valid = false;
        }
    }

    pub fn select_buffer_size(&mut self, size: u32) {
        let size = size
            .min(self.display_state.buffer_size_range.min)
            .max(self.display_state.buffer_size_range.max);
        if self.display_state.selected_buffer_size != size {
            self.display_state.selected_buffer_size = size;
        }
    }

    pub fn select_midi_server(&mut self, index: usize) {
        let index = index.min(self.display_state.midi_server_options.len() - 1);
        if self.display_state.selected_midi_server != index {
            self.display_state.selected_midi_server = index;

            self.display_state.midi_in_port_options = self.midi_server_info
                [self.display_state.selected_midi_server]
                .in_devices
                .iter()
                .map(|d| d.name.clone())
                .collect();
            self.display_state.midi_out_port_options = self.midi_server_info
                [self.display_state.selected_midi_server]
                .out_devices
                .iter()
                .map(|d| d.name.clone())
                .collect();

            self.set_defaults_for_current_midi_server();
        }
    }

    pub fn set_audio_defaults(&mut self) {
        self.display_state.selected_audio_server = self.default_audio_server;

        self.display_state.audio_device_options = self.audio_server_info
            [self.display_state.selected_audio_server]
            .devices
            .iter()
            .map(|d| d.name.clone())
            .collect();

        self.set_defaults_for_current_audio_server();
    }

    pub fn set_midi_defaults(&mut self) {
        self.display_state.selected_midi_server = self.default_midi_server;

        self.display_state.midi_in_port_options = self.midi_server_info
            [self.display_state.selected_midi_server]
            .in_devices
            .iter()
            .map(|d| d.name.clone())
            .collect();
        self.display_state.midi_out_port_options = self.midi_server_info
            [self.display_state.selected_midi_server]
            .out_devices
            .iter()
            .map(|d| d.name.clone())
            .collect();

        self.set_defaults_for_current_midi_server();
    }

    pub fn set_defaults_for_current_audio_server(&mut self) {
        self.display_state.is_valid = false;

        self.display_state.selected_audio_device =
            self.audio_server_info[self.display_state.selected_audio_server].default_device;

        self.set_defaults_for_current_audio_device();
    }

    pub fn set_defaults_for_current_audio_device(&mut self) {
        self.display_state.is_valid = false;

        self.display_state.audio_in_busses.clear();
        self.display_state.audio_out_busses.clear();

        if let Some(device) = self.audio_server_info[self.display_state.selected_audio_server]
            .devices
            .get(self.display_state.selected_audio_device)
        {
            self.display_state.audio_in_port_options = device.in_ports.clone();
            self.display_state.audio_out_port_options = device.out_ports.clone();

            self.display_state.sample_rate_options = device.sample_rates.clone();
            self.display_state.buffer_size_range = device.buffer_size_range;

            self.display_state.selected_sample_rate_index = device
                .default_sample_rate_index
                .min(self.display_state.sample_rate_options.len() - 1);
            self.display_state.selected_buffer_size = device
                .default_buffer_size
                .min(device.buffer_size_range.min)
                .max(device.buffer_size_range.max);

            if let Some(port) = device.in_ports.get(device.default_in_port) {
                self.display_state.audio_in_busses.push(AudioBus {
                    id: String::from("Mic In"),
                    system_ports: vec![port.clone()],
                });
            }

            if let Some(left_port) = device.out_ports.get(device.default_out_port_left) {
                if let Some(right_port) = device.out_ports.get(device.default_out_port_right) {
                    self.display_state.audio_out_busses.push(AudioBus {
                        id: String::from("Speaker Out"),
                        system_ports: vec![left_port.clone(), right_port.clone()],
                    });

                    // Only valid if there is atleast one output.
                    self.display_state.is_valid = true;
                }
            }
        } else {
            self.display_state.audio_in_port_options.clear();
            self.display_state.audio_out_port_options.clear();
            self.display_state.sample_rate_options.clear();
            self.display_state.buffer_size_range = BufferSizeRange::default();
        }
    }

    pub fn set_defaults_for_current_midi_server(&mut self) {
        self.display_state.midi_in_controllers.clear();
        self.display_state.midi_out_controllers.clear();

        if let Some(midi_in_port) = self
            .display_state
            .midi_in_port_options
            .get(self.midi_server_info[self.display_state.selected_midi_server].default_in_port)
        {
            self.display_state.midi_in_controllers.push(MidiController {
                id: String::from("Midi In"),
                system_port: midi_in_port.clone(),
            });
        }
    }

    pub fn display_state(&self) -> &DisplayState {
        &self.display_state
    }
}

#[derive(Debug, Clone)]
pub(crate) struct AudioDeviceInfo {
    pub name: String,
    pub in_ports: Vec<String>,
    pub out_ports: Vec<String>,
    pub sample_rates: Vec<u32>,
    pub buffer_size_range: BufferSizeRange,

    pub default_in_port: usize,
    pub default_out_port_left: usize,
    pub default_out_port_right: usize,
    pub default_sample_rate_index: usize,
    pub default_buffer_size: u32,
}

#[derive(Debug, Clone)]
pub(crate) struct AudioServerInfo {
    pub name: String,
    pub version: Option<String>,
    pub devices: Vec<AudioDeviceInfo>,
    pub available: bool,

    pub default_device: usize,
}

impl AudioServerInfo {
    pub(crate) fn new(name: String, version: Option<String>) -> Self {
        Self {
            name,
            version,
            devices: Vec::new(),
            available: false,
            default_device: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct MidiDeviceInfo {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct MidiServerInfo {
    pub name: String,
    pub version: Option<String>,
    pub in_devices: Vec<MidiDeviceInfo>,
    pub out_devices: Vec<MidiDeviceInfo>,
    pub available: bool,

    pub default_in_port: usize,
}

impl MidiServerInfo {
    pub(crate) fn new(name: String, version: Option<String>) -> Self {
        Self {
            name,
            version,
            in_devices: Vec::new(),
            out_devices: Vec::new(),
            available: false,
            default_in_port: 0,
        }
    }
}
