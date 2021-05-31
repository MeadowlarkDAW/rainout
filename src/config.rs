use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum UseDevice {
    Auto,
    Name(String),
    None,
}

impl UseDevice {
    pub fn get_name_or<'a>(&'a self, default: &'a str) -> &'a str {
        match self {
            UseDevice::Auto => default,
            UseDevice::Name(n) => n.as_str(),
            UseDevice::None => "",
        }
    }
}

impl Default for UseDevice {
    fn default() -> Self {
        UseDevice::Auto
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum UseChannels {
    /// Use up to the first two channels in this device.
    ///
    /// If the device is mono, then only the single channels will be used.
    ///
    /// If the device has more than two channels, then only the first two channels will be used.
    UpToFirstTwo,

    /// Use all the channels in the device.
    All,

    /// Use these specific channels (array of channel indexes).
    Use(Vec<u16>),
}

impl UseChannels {
    pub fn as_channel_index_array(&self, max_device_channels: u16) -> Vec<u16> {
        match self {
            UseChannels::UpToFirstTwo => {
                if max_device_channels == 1 {
                    vec![0]
                } else {
                    vec![0, 1]
                }
            }
            UseChannels::All => (0..max_device_channels).collect(),
            UseChannels::Use(v) => v.clone(),
        }
    }
}

impl Default for UseChannels {
    fn default() -> Self {
        UseChannels::UpToFirstTwo
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct AudioDeviceConfig {
    /// The ID to use for this device. This ID is for the "internal" device that appears to the user
    /// as list of available sources/sends. This is not necessarily the same as the name of the actual
    /// system hardware device that this "internal" device is connected to.
    ///
    /// This ID *must* be unique for each `AudioDeviceConfig` and `MidiDeviceConfig`.
    ///
    /// Examples of IDs can include:
    ///
    /// * Realtek Device In
    /// * Drums Mic
    /// * Headphones Out
    /// * Speakers Out
    pub id: String,

    /// The channels (of the system device) that this device will be connected to.
    pub system_channels: UseChannels,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct AudioServerConfig {
    /// The name of the audio server to use.
    pub server: String,

    /// The name of the system duplex audio device to use.
    ///
    /// Set this to `None` to use the default system device.
    pub system_duplex_device: Option<String>,

    /// The name of the system input device to use.
    ///
    /// This must be a child of the given `system_duplex_device`.
    pub system_in_device: UseDevice,

    // The name of the system output device to use.
    ///
    /// This must be a child of the given `system_duplex_device`.
    pub system_out_device: UseDevice,

    /// The audio input devices to create/use. These devices are the "internal" devices that appears to the user
    /// as list of available sources/sends. This is not necessarily the same as the actual
    /// system hardware device that these "internal" devices are connected to.
    ///
    /// Examples of device IDs can include:
    ///
    /// * Realtek Device In
    /// * Built-In Mic
    /// * Drums Mic
    pub create_in_devices: Vec<AudioDeviceConfig>,

    /// The audio output devices to create/use. These devices are the "internal" devices that appears to the user
    /// as list of available sources/sends. This is not necessarily the same as the actual
    /// system hardware device that these "internal" devices are connected to.
    ///
    /// Examples of IDs can include:
    ///
    /// * Realtek Device Stereo Out
    /// * Headphones Out
    /// * Speakers Out
    pub create_out_devices: Vec<AudioDeviceConfig>,

    /// The sample rate to use.
    ///
    /// Set this to `None` to use the default sample rate of the system device.
    pub sample_rate: Option<u32>,

    /// The maximum number of frames per channel.
    ///
    /// Set this to `None` to use the default settings of the system device.
    pub max_buffer_size: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct MidiDeviceConfig {
    /// The ID to use for this device. This ID is for the "internal" device that appears to the user
    /// as list of available sources/sends. This is not necessarily the same as the name of the actual
    /// system hardware device that this "internal" device is connected to.
    ///
    /// This ID *must* be unique for each `AudioDeviceConfig` and `MidiDeviceConfig`.
    pub id: String,

    /// The name of the system port this device is connected to.
    pub system_port: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct MidiServerConfig {
    /// The name of the audio server to use.
    pub server: String,

    /// The midi input devices to create/use. These devices are the "internal" devices that appears to the user
    /// as list of available sources/sends. This is not necessarily the same as the actual
    /// system hardware devices that these "internal" devices are connected to.
    pub create_in_devices: Vec<MidiDeviceConfig>,

    /// The midi output devices to create/use. These devices are the "internal" devices that appears to the user
    /// as list of available sources/sends. This is not necessarily the same as the actual
    /// system hardware devices that these "internal" devices are connected to.
    pub create_out_devices: Vec<MidiDeviceConfig>,
}
