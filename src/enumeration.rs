use crate::platform;

/// Returns the list available audio backends for this platform.
///
/// These are ordered with the first item (index 0) being the most highly
/// preferred default backend.
pub fn available_audio_backends() -> &'static [&'static str] {
    platform::available_audio_backends()
}

#[cfg(feature = "midi")]
/// Returns the list available midi backends for this platform.
///
/// These are ordered with the first item (index 0) being the most highly
/// preferred default backend.
pub fn available_midi_backends() -> &'static [&'static str] {
    platform::available_midi_backends()
}

/// Returns the list of available audio devices for the given backend.
///
/// This will return an error if the backend with the given name could
/// not be found.
pub fn enumerate_audio_backend(backend: &str) -> Result<AudioBackendOptions, ()> {
    platform::enumerate_audio_backend(backend)
}

/// Returns the configuration options for the given device.
///
/// This will return an error if the backend or the device could not
/// be found.
pub fn enumerate_audio_device(
    backend: &str,
    device: &DeviceID,
) -> Result<AudioDeviceConfigOptions, ()> {
    platform::enumerate_audio_device(backend, device)
}

#[cfg(any(feature = "jack-linux", feature = "jack-macos", feature = "jack-windows"))]
/// Returns the configuration options for "monolithic" system-wide Jack
/// audio device.
///
/// This will return an error if Jack is not installed on the system
/// or if the Jack server is not running.
pub fn enumerate_jack_audio_device(
) -> Result<JackAudioDeviceOptions, crate::error::JackEnumerationError> {
    #[cfg(target_os = "linux")]
    {
        #[cfg(feature = "jack-linux")]
        return platform::enumerate_jack_audio_device();

        #[cfg(not(feature = "jack-linux"))]
        return Err(crate::error::JackEnumerationError::NotEnabledForPlatform);
    }

    #[cfg(target_os = "macos")]
    {
        #[cfg(feature = "jack-macos")]
        return platform::enumerate_jack_audio_device();

        #[cfg(not(feature = "jack-macos"))]
        return Err(crate::error::JackEnumerationError::NotEnabledForPlatform);
    }

    #[cfg(target_os = "windows")]
    {
        #[cfg(feature = "jack-windows")]
        return platform::enumerate_jack_audio_device();

        #[cfg(not(feature = "jack-windows"))]
        return Err(crate::error::JackEnumerationError::NotEnabledForPlatform);
    }
}

#[cfg(feature = "asio")]
#[cfg(target_os = "windows")]
/// Returns the configuration options for the given ASIO device.
///
/// This will return an error if the device could not be found.
pub fn enumerate_asio_audio_device(device: &DeviceID) -> Result<AsioAudioDeviceOptions, ()> {
    platform::enumerate_asio_audio_device(device)
}

#[cfg(feature = "midi")]
/// Returns the list of available midi devices for the given backend.
///
/// This will return an error if the backend with the given name could
/// not be found.
pub fn enumerate_midi_backend(backend: &str) -> Result<MidiBackendOptions, ()> {
    platform::enumerate_midi_backend(backend)
}

#[derive(Debug, Clone)]
/// Information about an audio backend, including its available devices
/// and configurations
pub struct AudioBackendOptions {
    /// The name of this audio backend
    pub name: &'static str,

    /// The version of this audio backend (if that information is available)
    pub version: Option<String>,

    /// The available audio devices to select from
    pub device_options: AudioDeviceOptions,
}

#[derive(Debug, Clone)]
/// The available audio devices to select from
pub enum AudioDeviceOptions {
    /// Only a single audio device can be selected from this list. These
    /// devices may be output only, input only, or (most commonly)
    /// duplex.
    SingleDeviceOnly {
        /// The available audio devices to select from.
        options: Vec<DeviceID>,
    },

    /// A single input and output device pair can be selected from this list.
    LinkedInOutDevice {
        /// The names/IDs of the available input devices to select from
        input_devices: Vec<DeviceID>,
        /// The names/IDs of the available output devices to select from
        output_devices: Vec<DeviceID>,

        /// The available configurations for this device pair
        config_options: AudioDeviceConfigOptions,
    },

    #[cfg(any(feature = "jack-linux", feature = "jack-macos", feature = "jack-windows"))]
    /// There is a single "monolithic" system-wide Jack audio device
    JackSystemWideDevice,

    #[cfg(feature = "asio")]
    #[cfg(target_os = "windows")]
    /// A single ASIO device can be selected from this list.
    SingleAsioDevice {
        /// A single ASIO device can be selected from this list.
        options: Vec<DeviceID>,
    },
}

#[cfg(feature = "serde-config")]
#[derive(Debug, Clone, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
/// The name/ID of a device
pub struct DeviceID {
    /// The name of the device
    pub name: String,

    /// The unique identifier of this device (if one is available). This
    /// is usually more reliable than just the name of the device.
    pub identifier: Option<String>,
}

#[cfg(not(feature = "serde-config"))]
#[derive(Debug, Clone, PartialEq, Hash)]
/// The name/ID of a device
pub struct DeviceID {
    /// The name of the device
    pub name: String,

    /// The unique identifier of this device (if one is available). This
    /// is usually more reliable than just the name of the device.
    pub identifier: Option<String>,
}

#[derive(Debug, Clone)]
/// The available configuration options for the audio device/devices
pub struct AudioDeviceConfigOptions {
    /// The available sample rates to choose from.
    ///
    /// If the available sample rates could not be determined at this time,
    /// then this will be `None`.
    pub sample_rates: Option<Vec<u32>>,

    /// The available range of fixed block/buffer sizes
    ///
    /// If the device does not support fixed block/buffer sizes, then this
    /// will be `None`.
    pub block_sizes: Option<BlockSizeRange>,

    /// The number of input audio ports available
    pub num_input_ports: usize,
    /// The number of output audio ports available
    pub num_output_ports: usize,

    /// The layout of the input audio ports
    pub input_channel_layout: ChannelLayout,
    /// The layout of the output audio ports
    pub output_channel_layout: ChannelLayout,

    /// If `true` then it means that the application can request to take
    /// exclusive access of the device to improve latency.
    ///
    /// This is only relevant for WASAPI on Windows. This will always be
    /// `false` on other backends and platforms.
    pub can_take_exclusive_access: bool,
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
/// The channel layout of the audio ports
pub enum ChannelLayout {
    /// The device has not specified the channel layout of the audio ports
    Unspecified,
    /// The device has a single mono channel
    Mono,
    /// The device has multiple mono channels (i.e. multiple microphone
    /// inputs)
    MultiMono,
    /// The device has a single stereo channel
    Stereo,
    /// The device has multiple stereo channels
    MultiStereo,
    /// The special (but fairly common) case where the device has two stereo
    /// output channels: one for speakers and one for headphones
    StereoX2SpeakerHeadphone,
    /// Some other configuration not listed.
    Other(String),
    // TODO: More channel layouts
}

/// The range of possible block sizes for an audio device.
#[derive(Debug, Clone)]
pub struct BlockSizeRange {
    /// The minimum buffer/block size that can be used (inclusive)
    pub min: u32,

    /// The maximum buffer/block size that can be used (inclusive)
    pub max: u32,

    /// The default buffer/block size for this device
    pub default: u32,
}

#[cfg(any(feature = "jack-linux", feature = "jack-macos", feature = "jack-windows"))]
#[derive(Debug, Clone)]
/// Information and configuration options for the "monolithic" system-wide
/// Jack audio device
pub struct JackAudioDeviceOptions {
    /// If this is `false`, then it means that Jack is not installed on the
    /// system and thus cannot be used.
    pub installed_on_sytem: bool,

    /// If this is `false`, then it means that Jack is installed but it is
    /// not currently running on the system, and thus cannot be used until
    /// the Jack server is started.
    pub running: bool,

    /// The sample rate of the Jack device
    pub sample_rate: u32,

    /// The block size of the Jack device
    pub block_size: u32,

    /// The names of the available input ports to select from
    pub input_ports: Vec<String>,
    /// The names of the available output ports to select from
    pub output_ports: Vec<String>,

    /// The indexes of the default input ports, along with their channel
    /// layout.
    ///
    /// If no default input ports could be found, then this will be `None`.
    pub default_input_ports: Option<(Vec<usize>, ChannelLayout)>,
    /// The indexes of the default output ports, along with their channel
    /// layout.
    ///
    /// If no default output ports could be found, then this will be `None`.
    pub default_output_ports: Option<(Vec<usize>, ChannelLayout)>,
}

#[cfg(feature = "asio")]
#[cfg(target_os = "windows")]
#[derive(Debug, Clone)]
/// Information and configuration options for an ASIO audio device on
/// Windows
pub struct AsioAudioDeviceOptions {
    /// The configuration options for this ASIO audio device
    pub config_options: AudioDeviceConfigOptions,

    /// The path the the executable that launches the settings GUI for
    /// this ASIO device
    pub settings_application: std::path::PathBuf,
}

#[cfg(feature = "midi")]
#[derive(Debug, Clone)]
/// Information about a MIDI backend, including its available devices
/// and configurations
pub struct MidiBackendOptions {
    /// The name of this MIDI backend
    pub name: &'static str,

    /// The version of this MIDI backend (if that information is available)
    pub version: Option<String>,

    /// The names of the available input MIDI devices to select from
    pub in_device_ports: Vec<MidiDevicePortOptions>,
    /// The names of the available output MIDI devices to select from
    pub out_device_ports: Vec<MidiDevicePortOptions>,

    /// The index of the default/preferred input MIDI port for the backend
    ///
    /// This will be `None` if no default input port could be
    /// determined.
    pub default_in_port: Option<usize>,
    /// The index of the default/preferred output MIDI port for the backend
    ///
    /// This will be `None` if no default output port could be
    /// determined.
    pub default_out_port: Option<usize>,
}

#[cfg(feature = "midi")]
#[derive(Debug, Clone)]
/// Information and configuration options for a MIDI device port
pub struct MidiDevicePortOptions {
    /// The name/ID of this device
    pub id: DeviceID,

    /// The index of this port for this device
    pub port_index: usize,

    /// The type of control scheme that this port uses
    pub control_type: MidiControlScheme,
}

#[cfg(feature = "midi")]
#[cfg(feature = "serde-config")]
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
/// The type of control scheme that this port supports
pub enum MidiControlScheme {
    /// Supports only MIDI version 1
    Midi1,

    #[cfg(feature = "midi2")]
    /// Supports MIDI version 2 (and by proxy also supports MIDI version 1)
    Midi2,
    // TODO: Midi versions inbetween 1.0 and 2.0?
    // TODO: OSC devices?
}

#[cfg(feature = "midi")]
#[cfg(not(feature = "serde-config"))]
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq)]
/// The type of control scheme that this port supports
pub enum MidiControlScheme {
    /// Supports only MIDI version 1
    Midi1,

    #[cfg(feature = "midi2")]
    /// Supports MIDI version 2 (and by proxy also supports MIDI version 1)
    Midi2,
    // TODO: Midi versions inbetween 1.0 and 2.0?
    // TODO: OSC devices?
}

impl Default for MidiControlScheme {
    fn default() -> Self {
        MidiControlScheme::Midi1
    }
}
