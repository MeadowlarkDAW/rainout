use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum SystemChannels {
    /// Use only up to the first two channels in the system device.
    UpToStereo,

    /// Use all channels in the system device.
    All,

    /// Use these specific channels in the system device.
    ///
    /// i.e. `vec![0, 1, 6, 7]` = use channels 0, 1, 6, and 7.
    ///
    /// This will produce an error if the channel does not exist in the device.
    Use(Vec<u16>),
}

impl SystemChannels {
    pub fn as_index_vec(&self, system_device_channels: u16) -> Vec<u16> {
        match self {
            SystemChannels::UpToStereo => (0..system_device_channels.min(2)).collect(),
            SystemChannels::All => (0..system_device_channels).collect(),
            SystemChannels::Use(s) => s.clone(),
        }
    }
}

impl Default for SystemChannels {
    fn default() -> Self {
        SystemChannels::UpToStereo
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

    /// The name of the system device this device is connected to.
    pub system_device: String,

    /// The channels (of the system device) that this device will be connected to.
    pub system_channels: SystemChannels,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct AudioServerConfig {
    /// The name of the audio server to use.
    pub server_name: String,

    /// The audio input devices to create/use. These devices are the "internal" devices that appears to the user
    /// as list of available sources/sends. This is not necessarily the same as the actual
    /// system hardware devices that these "internal" devices are connected to.
    ///
    /// Examples of device IDs can include:
    ///
    /// * Realtek Device In
    /// * Built-In Mic
    /// * Drums Mic
    pub use_in_devices: Vec<AudioDeviceConfig>,

    /// The audio output devices to create/use. These devices are the "internal" devices that appears to the user
    /// as list of available sources/sends. This is not necessarily the same as the actual
    /// system hardware devices that these "internal" devices are connected to.
    ///
    /// Examples of IDs can include:
    ///
    /// * Realtek Device Stereo Out
    /// * Headphones Out
    /// * Speakers Out
    pub use_out_devices: Vec<AudioDeviceConfig>,

    /// The sample rate to use.
    ///
    /// Set this to `None` to use the default sample-rate of the audio server.
    pub use_sample_rate: Option<u32>,

    /// The maximum number of frames per channel.
    ///
    /// Set this to `None` to use the default settings of the audio server.
    pub use_max_buffer_size: Option<u32>,
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
    pub server_name: String,

    /// The midi input devices to create/use. These devices are the "internal" devices that appears to the user
    /// as list of available sources/sends. This is not necessarily the same as the actual
    /// system hardware devices that these "internal" devices are connected to.
    pub use_in_devices: Vec<MidiDeviceConfig>,

    /// The midi output devices to create/use. These devices are the "internal" devices that appears to the user
    /// as list of available sources/sends. This is not necessarily the same as the actual
    /// system hardware devices that these "internal" devices are connected to.
    pub use_out_devices: Vec<MidiDeviceConfig>,
}
