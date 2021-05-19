#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "linux")]
pub use linux::*;

#[derive(Debug, Clone, Copy)]
pub enum BufferSizeConfigs {
    ConstantSize {
        min_buffer_size: u32,
        max_buffer_size: u32,
    },
    UnknownSize,
}

#[derive(Debug, Clone)]
pub struct AudioDeviceAvailableConfigs {
    pub sample_rates: Vec<u32>,

    pub min_output_channels: u16,
    pub max_output_channels: u16,

    pub min_input_channels: u16,
    pub max_input_channels: u16,

    pub buffer_size: BufferSizeConfigs,
}

#[derive(Debug, Clone)]
pub struct AudioDeviceConfig {
    pub(crate) name: String,
    pub(crate) selected: bool,

    available_configs: AudioDeviceAvailableConfigs,

    /// The sample rate to use. Set this to `None` to use the default settings.
    sample_rate: Option<u32>,

    /// The number of output channels to use. Set this to `None` to use the default settings.
    output_channels: Option<u16>,

    /// The number of input channels to use. Set this to `None` to use the default settings.
    input_channels: Option<u16>,

    /// The buffer size in frames. Set this to `None` to use the default settings.
    buffer_size: Option<u32>,
}

impl AudioDeviceConfig {
    pub(crate) fn new(name: String, available_configs: AudioDeviceAvailableConfigs) -> Self {
        Self {
            name,
            selected: false,

            available_configs,

            sample_rate: None,
            output_channels: None,
            input_channels: None,
            buffer_size: None,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    /// The configurations that are available in this device
    pub fn available_configs(&self) -> &AudioDeviceAvailableConfigs {
        &self.available_configs
    }

    /// Set the sample rate to use.
    ///
    /// Set this to `None` to use the default settings. The default settings will vary per platform/audio server/device.
    ///
    /// If given an invalid input that is out of the range given by `available_configs()`, then it will be ignored and
    /// nothing will be changed.
    pub fn set_sample_rate(&mut self, sample_rate: Option<u32>) {
        if let Some(sample_rate) = sample_rate {
            if !self.available_configs.sample_rates.contains(&sample_rate) {
                return;
            }
        }

        self.sample_rate = sample_rate;
    }

    /// Set the number of output channels to use.
    ///
    /// Set this to `None` to use the default settings. The default settings will vary per platform/audio server/device.
    ///
    /// If given an invalid input that is out of the range given by `available_configs()`, then it will be ignored and
    /// nothing will be changed.
    pub fn set_output_channels(&mut self, output_channels: Option<u16>) {
        if let Some(output_channels) = output_channels {
            if output_channels < self.available_configs.min_output_channels
                || output_channels > self.available_configs.max_output_channels
            {
                return;
            }
        }

        self.output_channels = output_channels;
    }

    /// Set the number of input channels to use.
    ///
    /// Set this to `None` to use the default settings. The default settings will vary per platform/audio server/device.
    ///
    /// If given an invalid input that is out of the range given by `available_configs()`, then it will be ignored and
    /// nothing will be changed.
    pub fn set_input_channels(&mut self, input_channels: Option<u16>) {
        if let Some(input_channels) = input_channels {
            if input_channels < self.available_configs.min_input_channels
                || input_channels > self.available_configs.max_input_channels
            {
                return;
            }
        }

        self.input_channels = input_channels;
    }

    /// Set the buffer size (in frames) to use.
    ///
    /// Set this to `None` to use the default settings. The default settings will vary per platform/audio server/device.
    ///
    /// If given an invalid input that is out of the range given by `available_configs()`, then it will be ignored and
    /// nothing will be changed.
    pub fn set_buffer_size(&mut self, buffer_size: Option<u32>) {
        if let Some(buffer_size) = buffer_size {
            match self.available_configs.buffer_size {
                BufferSizeConfigs::ConstantSize {
                    min_buffer_size,
                    max_buffer_size,
                } => {
                    if buffer_size < min_buffer_size || buffer_size > max_buffer_size {
                        return;
                    }
                }
                BufferSizeConfigs::UnknownSize => {
                    return;
                }
            }
        }

        self.buffer_size = buffer_size;
    }

    /// The sample rate to use. This will return `None` if using the default settings.
    pub fn sample_rate(&self) -> Option<u32> {
        self.sample_rate
    }

    /// The number of output channels to use. This will return `None` if using the default settings.
    pub fn output_channels(&self) -> Option<u16> {
        self.output_channels
    }

    /// The number of input channels to use. This will return `None` if using the default settings.
    pub fn input_channels(&self) -> Option<u16> {
        self.input_channels
    }

    /// The buffer size to use (in frames). This will return `None` if using the default settings.
    pub fn buffer_size(&self) -> Option<u32> {
        self.buffer_size
    }

    pub fn set_selected(&mut self, selected: bool) {
        self.selected = selected;
    }
    pub fn selected(&self) -> bool {
        self.selected
    }

    pub(crate) fn update_available_configs(
        &mut self,
        available_configs: AudioDeviceAvailableConfigs,
    ) {
        self.available_configs = available_configs;

        // Make sure that the existing config is still valid
        if let Some(sample_rate) = self.sample_rate {
            if !self.available_configs.sample_rates.contains(&sample_rate) {
                self.sample_rate = None;
            }
        }
        if let Some(output_channels) = self.output_channels {
            if output_channels < self.available_configs.min_output_channels
                || output_channels > self.available_configs.max_output_channels
            {
                self.output_channels = None;
            }
        }
        if let Some(input_channels) = self.input_channels {
            if input_channels < self.available_configs.min_input_channels
                || input_channels > self.available_configs.max_input_channels
            {
                self.input_channels = None;
            }
        }
        if let Some(buffer_size) = self.buffer_size {
            match self.available_configs.buffer_size {
                BufferSizeConfigs::ConstantSize {
                    min_buffer_size,
                    max_buffer_size,
                } => {
                    if buffer_size < min_buffer_size || buffer_size > max_buffer_size {
                        self.buffer_size = None;
                    }
                }
                BufferSizeConfigs::UnknownSize => {
                    self.buffer_size = None;
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct MidiDeviceAvailableConfigs {
    // TODO
}

pub struct MidiDevice {
    // TODO
}

#[derive(Debug, Clone)]
pub struct AudioServerConfig {
    pub(crate) name: String,
    pub(crate) version: Option<String>,
    pub(crate) devices: Vec<AudioDeviceConfig>,
    pub(crate) active: bool,
    pub(crate) selected: bool,
}

impl AudioServerConfig {
    pub(crate) fn new(name: String, version: Option<String>) -> Self {
        Self {
            name,
            version,
            devices: Vec::new(),
            active: false,
            selected: false,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn version(&self) -> &Option<String> {
        &self.version
    }

    pub fn audio_devices(&self) -> &[AudioDeviceConfig] {
        &self.devices
    }
    pub fn audio_devices_mut(&mut self) -> &mut [AudioDeviceConfig] {
        &mut self.devices
    }

    pub fn active(&self) -> bool {
        self.active
    }

    pub fn set_selected(&mut self, selected: bool) {
        self.selected = selected;
    }
    pub fn selected(&self) -> bool {
        self.selected
    }
}

pub trait RtProcessHandler: 'static + Send + Sized {
    fn process(&mut self, proc_info: ProcessInfo);
}

pub struct ProcessInfo<'a> {
    pub audio_inputs: &'a [Vec<f32>],
    pub audio_outputs: &'a mut [Vec<f32>],

    pub audio_in_channels: u16,
    pub audio_out_channels: u16,
    pub audio_frames: usize,

    pub sample_rate: u32,
    // TODO: MIDI IO
}

#[derive(Debug, Clone, Copy)]
pub struct EstimatedLatency {
    pub frames: u32,
    pub sample_rate: u32,
}

impl EstimatedLatency {
    pub fn as_duration(&self) -> std::time::Duration {
        std::time::Duration::from_secs_f64(f64::from(self.frames) / f64::from(self.sample_rate))
    }
}

impl<'a> std::fmt::Debug for ProcessInfo<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProcessInfo")
            .field("audio_in_channels", &self.audio_in_channels)
            .field("audio_out_channels", &self.audio_out_channels)
            .field("audio_frames", &self.audio_frames)
            .field("sample_rate", &self.sample_rate)
            .finish()
    }
}

#[derive(Debug)]
pub enum SpawnRtThreadError {
    NoAudioServerSelected,
    NoAudioDeviceSelected(String),
    AudioServerUnavailable(String),
    PlatformSpecific(Box<dyn std::error::Error + Send + 'static>),
}

impl std::error::Error for SpawnRtThreadError {}

impl std::fmt::Display for SpawnRtThreadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpawnRtThreadError::NoAudioServerSelected => {
                write!(f, "Error spawning rt thread: No audio server was selected.")
            }
            SpawnRtThreadError::NoAudioDeviceSelected(server) => {
                write!(
                    f,
                    "Error spawning rt thread: No audio device was selected for server {:?}.",
                    server
                )
            }
            SpawnRtThreadError::AudioServerUnavailable(server) => {
                write!(
                    f,
                    "Error spawning rt thread: The audio sever is unavailable: {:?}.",
                    server
                )
            }
            SpawnRtThreadError::PlatformSpecific(e) => {
                write!(f, "Error spawning rt thread: Platform error: {:?}", e)
            }
        }
    }
}

#[derive(Debug)]
pub enum StreamError {
    AudioServerDisconnected(String),
    AudioDeviceDisconnected(String),
    PlatformSpecific(Box<dyn std::error::Error + Send + 'static>),
}

impl std::error::Error for StreamError {}

impl std::fmt::Display for StreamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StreamError::AudioServerDisconnected(server) => {
                write!(
                    f,
                    "Stream error: The audio sever was disconnected: {:?}.",
                    server
                )
            }
            StreamError::AudioDeviceDisconnected(device) => {
                write!(
                    f,
                    "Stream error: The audio device was disconnected: {:?}.",
                    device
                )
            }
            StreamError::PlatformSpecific(e) => {
                write!(f, "Stream error: Platform error: {:?}", e)
            }
        }
    }
}
