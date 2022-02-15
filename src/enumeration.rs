use crate::platform;

/// Retrieve the list of available audio backends on the system, along with
/// the available devices for each backend.
///
/// (i.e. Jack, Pipewire, WASAPI, CoreAudio, etc.)
///
/// Calling these a second time will essentially "refresh" the list of available
/// devices.
pub fn audio_backends() -> Vec<AudioBackendInfo> {
    platform::audio_backends()
}

/// Retrieve the list of available midi backends on the system, along with
/// the available devices for each backend.
///
/// (i.e. Jack, Pipewire, WASAPI, CoreAudio, etc.)
///
/// Calling these a second time will essentially "refresh" the list of available
/// devices.
pub fn midi_backends() -> Vec<MidiBackendInfo> {
    platform::midi_backends()
}

/// Information about a particular audio backend, including a list of the
/// available devices.
#[derive(Debug, Clone)]
pub struct AudioBackendInfo {
    /// The name of this backend (i.e. Jack, Pipewire, WASAPI, CoreAudio, etc.)
    pub name: String,

    /// If true, then it means this backend is the default/preferred backend for
    /// the given system. Only one item in the list from `audio_backends()` will
    /// have this set to true.
    pub is_default: bool,

    /// The version of this backend (if there is one available)
    ///
    /// (i.e. "1.2.10")
    pub version: Option<String>,

    /// The devices that are available in this backend.
    ///
    /// Please note that these are not necessarily each physical device in the
    /// system. For example, in backends like Jack and CoreAudio, the whole system
    /// acts like a single "duplex device" which is the audio server.
    pub devices: Vec<AudioDeviceInfo>,

    /// If this is true, then it means it is relevant to actually show the available
    /// devices as a list to select from in a settings GUI.
    ///
    /// In backends like Jack and CoreAudio which set this to false, there is only
    /// ever one "system-wide duplex device" which is the audio server itself, and
    /// showing this information in a settings GUI is irrelevant.
    pub devices_are_relevant: bool,

    /// If this is true, then it means this backend is available and running on
    /// this system. (For example, if this backend is Jack and the Jack server is
    /// not currently running on the system, then this will be false.)
    pub available: bool,
}

/// Information about a particular audio device, including all its available
/// configurations.
#[derive(Debug, Clone)]
pub struct AudioDeviceInfo {
    /// The name of this device.
    ///
    /// Note if there are multiple devices with the same name then a number will
    /// be appended to it. (i.e. "Interface, "Interface #2")
    pub name: String,

    /// If true, then it means this device is the default/preferred device for
    /// this given backend. Only one device in the backend's list will have this set
    /// to true.
    pub is_default: bool,

    /// The names of the available input ports (one port per channel) on this device
    /// (i.e. "mic_1", "mic_2", "system_input", etc.)
    pub in_ports: Vec<String>,

    /// The names of the available output ports (one port per channel) on this device
    /// (i.e. "out_1", "speakers_out_left", "speakers_out_right", etc.)
    pub out_ports: Vec<String>,

    /// The available sample rates for this device.
    pub sample_rates: Vec<u32>,

    /// The default/preferred sample rate for this audio device.
    pub default_sample_rate: u32,

    /// The supported range of fixed buffer/block sizes for this device. If the device
    /// doesn't support fixed-size buffers then this will be `None`.
    pub fixed_buffer_size_range: Option<FixedBufferSizeRange>,

    /// The default channel layout of the input ports for this device.
    pub default_input_layout: DefaultChannelLayout,

    /// The default channel layout of the output ports for this device.
    pub default_output_layout: DefaultChannelLayout,
}

/// The range of possible fixed sizes of buffers/blocks for an audio device.
#[derive(Debug, Clone)]
pub struct FixedBufferSizeRange {
    /// The minimum buffer/block size (inclusive)
    pub min: u32,
    /// The maximum buffer/block size (inclusive)
    pub max: u32,

    /// If this is `true` then it means the device only supports fixed buffer/block
    /// sizes between `min` and `max` that are a power of 2.
    pub must_be_power_of_2: bool,

    /// The default/preferred fixed buffer size for this device.
    pub default: u32,
}

/// The default channel layout of the ports for an audio device.
///
/// These include the index of each port for each channel.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum DefaultChannelLayout {
    /// The device has not specified the default channel layout of its ports.
    Unspecified,

    Mono(usize),
    Stereo {
        left: usize,
        right: usize,
    },
    // TODO: More channel layouts
}

/// Information about a particular midi backend, including a list of the
/// available devices.
#[derive(Debug, Clone)]
pub struct MidiBackendInfo {
    /// The name of this backend (i.e. Jack, Pipewire, WASAPI, CoreAudio, etc.)
    pub name: String,

    /// If true, then it means this backend is the default/preferred backend for
    /// the given system. Only one item in the `midi_backends()` list will have
    /// set this to true.
    pub is_default: bool,

    /// The version of this backend (if there is one available)
    ///
    /// (i.e. "1.2.10")
    pub version: Option<String>,

    /// The list of available input MIDI devices
    pub in_devices: Vec<MidiDeviceInfo>,

    /// The list of available output MIDI devices
    pub out_devices: Vec<MidiDeviceInfo>,

    /// If this is true, then it means this backend is available and running on
    /// this system. (For example, if this backend is Jack and the Jack server is
    /// not currently running on the system, then this will be false.)
    pub available: bool,
}

/// Information about a particular midi device, including all its available
/// configurations.
#[derive(Debug, Clone)]
pub struct MidiDeviceInfo {
    // The name of this device
    pub name: String,

    // If true, then it means this device is the default/preferred device for
    // the given backend. Only one input and one output device in the backend's
    // list will have this set to true.
    pub is_default: bool,
}
