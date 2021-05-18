#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "linux")]
pub use linux::*;

#[derive(Debug, Clone, Copy)]
pub enum BlockSizeConfigs {
    ConstantSize {
        min_block_size: u32,
        max_block_size: u32,
    },
    UnknownSize
}

#[derive(Debug, Clone)]
pub struct AudioDeviceAvailableConfigs {
    sample_rates: Vec<u32>,

    min_output_channels: u16,
    max_output_channels: u16,

    min_input_channels: u16,
    max_input_channels: u16,

    block_size: BlockSizeConfigs,
}

#[derive(Debug, Copy, Clone)]
pub struct AudioDeviceConfig {
    available_configs: AudioDeviceAvailableConfigs,

    /// The sample rate to use. Set this to `None` to use the default settings.
    sample_rate: Option<u32>,

    /// The number of output channels to use. Set this to `None` to use the default settings.
    output_channels: Option<u16>,

    /// The number of input channels to use. Set this to `None` to use the default settings.
    input_channels: Option<u16>,

    /// The block size in frames. Set this to `None` to use the default settings.
    block_size: Option<u32>,
}

impl AudioDeviceConfig {
    pub(crate) fn new(available_configs: AudioDeviceAvailableConfigs) -> Self {
        Self {
            available_configs,
            sample_rate: None,
            output_channels: None,
            input_channels: None,
            block_size: None,
        }
    }

    /// Set the sample rate to use.
    ///
    /// Set this to `None` to use the default settings. The default settings will vary per platform/audio server/device.
    ///
    /// If given an invalid input that is out of the range given by `available_configs()`, then it will be ignored and
    /// nothing will be changed.
    pub fn set_sample_rate(&mut self, sample_rate: Option<u32>) {
        if let Some(sample_rate) = sample_rate {
            if !self.available_configs.sample_rates.contains(sample_rate) {
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
            if output_channels < self.available_configs.min_output_channels || output_channels > self.available_configs.max_output_channels {
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
            if input_channels < self.available_configs.min_input_channels || input_channels > self.available_configs.max_input_channels {
                return;
            }
        }

        self.input_channels = input_channels;
    }

    /// Set the block size (in frames) to use.
    ///
    /// Set this to `None` to use the default settings. The default settings will vary per platform/audio server/device.
    ///
    /// If given an invalid input that is out of the range given by `available_configs()`, then it will be ignored and
    /// nothing will be changed.
    pub fn set_block_size(&mut self, block_size: Option<u32>) {
        if let Some(block_size) = block_size {
            match self.available_configs.block_size {
                BlockSizeConfigs::ConstantSize { min_block_size, max_block_size } => {
                    if block_size < min_block_size || block_size > max_block_size {
                        return;
                    }
                }
                BlockSizeConfigs::UnknownSize => {
                    return;
                }
            }
        }

        self.block_size = block_size;
    }

    /// Returns the available configurations for this device
    pub fn available_configs(&self) -> &AudioDeviceAvailableConfigs {
        &self.available_configs
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

    /// The block size to use (in frames). This will return `None` if using the default settings.
    pub fn block_size(&self) -> Option<u32> {
        self.block_size
    }

    pub(crate) fn update_available_configs(&mut self, available_configs: AudioDeviceAvailableConfigs) {
        self.available_configs = available_configs;

        // Make sure that the existing config is still valid
        if let Some(sample_rate) = self.sample_rate {
            if !self.available_configs.sample_rates.contains(sample_rate) {
                self.sample_rate = None;
            }
        }
        if let Some(output_channels) = self.output_channels {
            if output_channels < self.available_configs.min_output_channels || output_channels > self.available_configs.max_output_channels {
                self.output_channels = None;
            }
        }
        if let Some(input_channels) = self.input_channels {
            if input_channels < self.available_configs.min_input_channels || input_channels > self.available_configs.max_input_channels {
                self.input_channels = None;
            }
        }
        if let Some(block_size) = self.block_size {
            match self.available_configs.block_size {
                BlockSizeConfigs::ConstantSize { min_block_size, max_block_size } => {
                    if block_size < min_block_size || block_size > max_block_size {
                        self.block_size = None;
                    }
                }
                BlockSizeConfigs::UnknownSize => {
                    self.block_size = None;
                }
            }
        }
    }
}

pub struct AudioDevice {
    name: String,
    config: AudioDeviceConfig,
    selected: bool,
}

impl AudioDevice {
    pub(crate) fn new(name: String, available_configs: AudioDeviceAvailableConfigs) -> Self {
        Self {
            name,
            config: AudioDeviceConfig::new(available_configs),
            selected: false,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn available_configs(&self) -> &AudioDeviceAvailableConfigs {
        self.config.available_configs()
    }

    pub fn config(&self) -> &AudioDeviceConfig {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut AudioDeviceConfig {
        &mut self.config
    }

    pub fn set_selected(&mut self, selected: bool) {
        self.selected = selected;
    }

    pub fn selected(&self) -> bool {
        self.selected
    }

    pub(crate) fn update_available_configs(&mut self, available_configs: AudioDeviceAvailableConfigs) {
        self.config.update_available_configs(available_configs);
    }
}

#[derive(Debug, Clone)]
pub struct MidiDeviceAvailableConfigs {
    // TODO
}

pub struct MidiDevice {
    // TODO
}

pub trait AudioServer {
    fn name(&self) -> &'static str;

    fn version(&self) -> Option<&str>;

    fn audio_devices(&mut self) -> &mut [AudioDevice];

    fn set_selected(&mut self, selected: bool);
    fn selected(&self) -> bool;
}