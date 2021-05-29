use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum UseU32 {
    Use(u32),
    Auto,
}

impl Default for UseU32 {
    fn default() -> Self {
        UseU32::Auto
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum UseName {
    Use(String),
    Auto,
}

impl UseName {
    pub fn get_name_or<'a>(&'a self, default: &'a str) -> &'a str {
        match self {
            UseName::Use(n) => n.as_str(),
            UseName::Auto => default,
        }
    }
}

impl Default for UseName {
    fn default() -> Self {
        UseName::Auto
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

    /// The channel indexes (of the system device) that this device will be connected to.
    pub system_channels: Vec<u16>,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct AudioServerConfig {
    /// The name of the audio server to use.
    pub server: UseName,

    /// The name of the system duplex audio device to use.
    pub system_duplex_device: UseName,

    /// The name of the system input device to use.
    ///
    /// This must be a child of the given `system_duplex_device`.
    pub system_in_device: UseName,

    // The name of the system output device to use.
    ///
    /// This must be a child of the given `system_duplex_device`.
    pub system_out_device: UseName,

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
    pub sample_rate: UseU32,

    /// The maximum number of frames per channel.
    pub max_buffer_size: UseU32,
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
    pub system_port: UseName,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct MidiServerConfig {
    /// The name of the audio server to use.
    pub server: UseName,

    /// The midi input devices to create/use. These devices are the "internal" devices that appears to the user
    /// as list of available sources/sends. This is not necessarily the same as the actual
    /// system hardware devices that these "internal" devices are connected to.
    pub create_in_devices: Vec<MidiDeviceConfig>,

    /// The midi output devices to create/use. These devices are the "internal" devices that appears to the user
    /// as list of available sources/sends. This is not necessarily the same as the actual
    /// system hardware devices that these "internal" devices are connected to.
    pub create_out_devices: Vec<MidiDeviceConfig>,
}
