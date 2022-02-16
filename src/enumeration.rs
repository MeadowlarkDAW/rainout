use crate::{platform, Config};

/// Returns the available audio backends for this platform.
pub fn available_audio_backends() -> &'static [AudioBackend] {
    platform::available_audio_backends()
}

#[cfg(feature = "midi")]
/// Returns the available midi backends for this platform.
pub fn available_midi_backends() -> &'static [MidiBackend] {
    platform::available_midi_backends()
}

/// Get information about a particular audio backend.
///
/// This will update the list of available devices as well as the the
/// status of whether or not this backend is running.
///
/// This will return an error if the backend is not available on this system.
pub fn enumerate_audio_backend(backend: AudioBackend) -> Result<AudioBackendInfo, ()> {
    platform::enumerate_audio_backend(backend)
}

/// Get information about a particular audio device.
///
/// This will return an error if the given device was not found.
pub fn enumerate_audio_device(
    backend: AudioBackend,
    device_id: &DeviceID,
) -> Result<AudioDeviceInfo, ()> {
    platform::enumerate_audio_device(backend, device_id)
}

#[cfg(feature = "midi")]
/// Get information about a particular midi backend.
///
/// This will update the list of available devices as well as the the
/// status of whether or not this backend is running.
///
/// This will return an error if the backend is not available on this system.
pub fn enumerate_midi_backend(backend: MidiBackend) -> Result<MidiBackendInfo, ()> {
    platform::enumerate_midi_backend(backend)
}

#[cfg(feature = "midi")]
/// Get information about a particular midi device.
///
/// This will return an error if the given device was not found.
pub fn enumerate_midi_device(
    backend: MidiBackend,
    device_id: &DeviceID,
) -> Result<MidiDeviceInfo, ()> {
    platform::enumerate_midi_device(backend, device_id)
}

/// Enumerate through each backend to find the preferred/best default audio
/// backend for this system.
///
/// If a higher priority backend does not have any available devices, then
/// this will try to return the next best backend that does have an
/// available device.
///
/// This does not enumerate through the devices in each backend, just the
/// names of each device.
pub fn find_preferred_audio_backend() -> AudioBackend {
    platform::find_preferred_audio_backend()
}

#[cfg(feature = "midi")]
/// Enumerate through each backend to find the preferred/best default midi
/// backend for this system.
///
/// If a higher priority backend does not have any available devices, then
/// this will try to return the next best backend that does have an
/// available device.
///
/// This does not enumerate through the devices in each backend, just the
/// names of each device.
pub fn find_preferred_midi_backend() -> MidiBackend {
    platform::find_preferred_midi_backend()
}

/// Enumerate through each audio device to find the preferred/best default audio
/// device for this backend.
///
/// This process can be slow. Try to use `AudioBackendInfo::preferred_device`
/// before calling this method.
pub fn find_preferred_audio_device(backend: AudioBackend) -> Option<AudioDeviceInfo> {
    platform::find_preferred_audio_device(backend)
}

#[cfg(feature = "midi")]
/// Enumerate through each midi device to find the preferred/best default midi
/// device for this backend.
///
/// This process can be slow. Try to use `MidiBackendInfo::preferred_in_device` and
/// `MidiBackendInfo::preferred_out_device` before calling this method.
pub fn find_preferred_midi_device(backend: MidiBackend) -> Option<MidiDeviceInfo> {
    platform::find_preferred_midi_device(backend)
}

/// Information about a particular audio backend, including a list of the
/// available devices.
#[derive(Debug, Clone)]
pub struct AudioBackendInfo {
    /// The type of backend.
    pub backend: AudioBackend,

    /// The version of this backend (if there is one available)
    ///
    /// (i.e. "1.2.10")
    pub version: Option<String>,

    /// If this is true, then it means this backend is running on this system.
    /// (For example, if this backend is Jack and the Jack server is not currently
    /// running on the system, then this will be false.)
    pub running: bool,

    /// The names/identifiers of the devices that are available in this backend.
    ///
    /// Please note that these are not necessarily each physical device in the
    /// system. For example, in backends like Jack and CoreAudio, the whole system
    /// acts like a single "duplex device" which is the audio server.
    pub devices: Vec<DeviceID>,

    /// The preferred/best default device for this backend.
    ///
    /// This will be `None` if the preferred device is not known at this time.
    pub preferred_device: Option<DeviceID>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeviceID {
    /// The name of this device.
    pub name: String,

    /// The unique identifier of this device (if one is available).
    ///
    /// This is usually more reliable than just using the name of
    /// the device.
    pub unique_id: Option<String>,
}

/// An audio backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioBackend {
    /// Pipewire on Linux
    Pipewire,

    #[cfg(feature = "jack-linux")]
    /// Jack on Linux
    JackLinux,

    /*
    #[cfg(feature = "alsa")]
    /// Alsa on Linux
    Alsa,

    #[cfg(feature = "pulseaudio")]
    /// Pulseaudio on Linux
    Pulseaudio,
    */
    /// CoreAudio on Mac
    CoreAudio,

    /*
    #[cfg(feature = "jack-macos")]
    /// Jack on MacOS
    JackMacOS,
    */
    /// WASAPI on Windows
    Wasapi,

    #[cfg(feature = "asio")]
    /// ASIO on Windows
    Asio,
    /*
    #[cfg(feature = "jack-windows")]
    /// Jack on Windows
    JackWindows,
    */
}

impl AudioBackend {
    /// If this is true, then it means it is relevant to actually show the available
    /// devices as a list to select from in a settings GUI.
    ///
    /// In backends like Jack and CoreAudio which set this to false, there is only
    /// ever one "system-wide duplex device" which is the audio server itself, and
    /// thus showing this information in a settings GUI is irrelevant.
    pub fn devices_are_relevant(&self) -> bool {
        match self {
            AudioBackend::Pipewire => false,

            #[cfg(feature = "jack-linux")]
            AudioBackend::JackLinux => false,

            /*
            #[cfg(feature = "alsa")]
            Backend::Alsa => true,

            #[cfg(feature = "pulseaudio")]
            Pulseaudio => true,
            */
            AudioBackend::CoreAudio => false,

            /*
            #[cfg(feature = "jack-macos")]
            Backend::JackMacOS => false,
            */
            AudioBackend::Wasapi => true,

            #[cfg(feature = "asio")]
            Backend::Asio => true,
            /*
            #[cfg(feature = "jack-windows")]
            Backend::JackWindows => false,
            */
        }
    }
}

#[cfg(feature = "midi")]
/// A midi backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MidiBackend {
    /// Pipewire on Linux
    Pipewire,

    #[cfg(feature = "jack-linux")]
    /// Jack on Linux
    JackLinux,

    /*
    #[cfg(feature = "alsa")]
    /// Alsa on Linux
    Alsa,

    #[cfg(feature = "pulseaudio")]
    /// Pulseaudio on Linux
    Pulseaudio,
    */
    /// CoreAudio on Mac
    CoreAudio,

    /*
    #[cfg(feature = "jack-macos")]
    /// Jack on MacOS
    JackMacOS,
    */
    /// WASAPI on Windows
    Wasapi,

    #[cfg(feature = "asio")]
    /// ASIO on Windows
    Asio,
    /*
    #[cfg(feature = "jack-windows")]
    /// Jack on Windows
    JackWindows,
    */
}

/// Information about a particular audio device, including all its available
/// configurations.
#[derive(Debug, Clone)]
pub struct AudioDeviceInfo {
    pub id: DeviceID,

    /// The names of the available input ports (one port per channel) on this device
    /// (i.e. "mic_1", "mic_2", "system_input", etc.)
    pub in_ports: Vec<String>,

    /// The names of the available output ports (one port per channel) on this device
    /// (i.e. "out_1", "speakers_out_left", "speakers_out_right", etc.)
    pub out_ports: Vec<String>,

    /// The available sample rates for this device.
    ///
    /// This is irrelevant for ASIO devices because the buffer size is configured
    /// through the configuration GUI application for that device.
    pub sample_rates: Vec<u32>,

    /// The default/preferred sample rate for this audio device.
    ///
    /// This is irrelevant for ASIO devices because the buffer size is configured
    /// through the configuration GUI application for that device.
    pub default_sample_rate: u32,

    /// The supported range of fixed buffer/block sizes for this device. If the device
    /// doesn't support fixed-size buffers then this will be `None`.
    ///
    /// This is irrelevant for ASIO devices because the buffer size is configured
    /// through the configuration GUI application for that device.
    pub fixed_buffer_size_range: Option<FixedBufferSizeRange>,

    /// The default channel layout of the input ports for this device.
    pub default_input_layout: DefaultChannelLayout,

    /// The default channel layout of the output ports for this device.
    pub default_output_layout: DefaultChannelLayout,

    #[cfg(feature = "asio")]
    /// If this audio device is an ASIO device, then this will contain extra
    /// information about the device.
    pub asio_info: Option<AsioDeviceInfo>,
}

#[cfg(feature = "asio")]
#[derive(Debug, Clone)]
pub struct AsioDeviceInfo {
    /// The path to the configuration GUI application for the device.
    pub config_gui_path: std::path::PathBuf,

    /// The sample rate that has been configured for this device.
    ///
    /// You will need to re-enumerate this device to get the new sample
    /// rate after configuring through the device's configuration GUI
    /// application.
    pub sample_rate: u32,

    /// The fixed buffer size that has been configured for this device.
    ///
    /// You will need to re-enumerate this device to get the new sample
    /// rate after configuring through the device's configuration GUI
    /// application.
    pub fixed_buffer_size: u32,
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

#[cfg(feature = "midi")]
/// Information about a particular midi backend, including a list of the
/// available devices.
#[derive(Debug, Clone)]
pub struct MidiBackendInfo {
    /// The type of backend.
    pub backend: MidiBackend,

    /// The version of this backend (if there is one available)
    ///
    /// (i.e. "1.2.10")
    pub version: Option<String>,

    /// If this is true, then it means this backend is running on this system.
    /// (For example, if this backend is Jack and the Jack server is not currently
    /// running on the system, then this will be false.)
    pub running: bool,

    /// The list of available input MIDI devices
    pub in_devices: Vec<DeviceID>,

    /// The list of available output MIDI devices
    pub out_devices: Vec<DeviceID>,

    /// The preferred/best default input device for this backend.
    ///
    /// This will be `None` if the preferred device is not known at this time.
    pub preferred_in_device: Option<DeviceID>,

    /// The preferred/best default output device for this backend.
    ///
    /// This will be `None` if the preferred device is not known at this time.
    pub preferred_out_device: Option<DeviceID>,
}

#[cfg(feature = "midi")]
/// Information about a particular midi device, including all its available
/// configurations.
#[derive(Debug, Clone)]
pub struct MidiDeviceInfo {
    pub id: DeviceID,
    // TODO: More information about the MIDI device
}
