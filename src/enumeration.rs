use crate::platform;

/// Returns the available audio backends for this platform.
///
/// These are ordered with the first item (index 0) being the most highly
/// preferred default backend.
pub fn available_audio_backends() -> &'static [AudioBackend] {
    platform::available_audio_backends()
}

#[cfg(feature = "midi")]
/// Returns the available midi backends for this platform.
///
/// These are ordered with the first item (index 0) being the most highly
/// preferred default backend.
pub fn available_midi_backends() -> &'static [MidiBackend] {
    platform::available_midi_backends()
}

/// Get information about a particular audio backend and its devices.
///
/// This will update the list of available devices as well as the the
/// status of whether or not this backend is running.
///
/// This will return an error if the backend is not available on this system.
pub fn enumerate_audio_backend(backend: AudioBackend) -> Result<AudioBackendInfo, ()> {
    platform::enumerate_audio_backend(backend)
}

#[cfg(feature = "midi")]
/// Get information about a particular midi backend and its devices.
///
/// This will update the list of available devices as well as the the
/// status of whether or not this backend is running.
///
/// This will return an error if the backend is not available on this system.
pub fn enumerate_midi_backend(backend: MidiBackend) -> Result<MidiBackendInfo, ()> {
    platform::enumerate_midi_backend(backend)
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

    /// The running status of this backend, along with all the available devices
    /// and their configurations.
    pub status: AudioBackendStatus,
}

impl AudioBackendInfo {
    /// The index of the preferred/default device.
    ///
    /// This will be `None` if the backend is not running or if the default
    /// device is not known.
    pub fn default_device_i(&self) -> Option<usize> {
        self.status.default_device_i()
    }

    /// Get the info of a particular audio device.
    ///
    /// This will be `None` if the backend is not running or if the given
    /// index is out of bounds.
    pub fn device_info(&self, index: usize) -> Option<&AudioDeviceInfo> {
        self.status.device_info(index)
    }
}

/// The running status of this backend, along with all the available devices
/// and their configurations.
#[derive(Debug, Clone)]
pub enum AudioBackendStatus {
    /// The audio backend is not running/or is not available.
    NotRunning,

    /// The audio backend is running, but no devices were found.
    RunningButNoDevices,

    /// The audio backend is running.
    ///
    /// Also, with this backend, showing the end-user a list of available devices
    /// is irrelevant because this backend only ever has one default "system-wide"
    /// device that is always selected.
    ///
    /// For example, in backends like Jack and CoreAudio, the whole system
    /// acts like a single "duplex device" which is the audio server itself.
    RunningWithSystemWideDevice(AudioDeviceInfo),

    /// The audio backend is running with a list of available devices to choose from.
    Running {
        /// The devices that are available in this backend.
        ///
        /// Please note that these are not necessarily each physical device in the
        /// system. For example, in backends like Jack and CoreAudio, the whole system
        /// acts like a single "duplex device" which is the audio server itself.
        devices: Vec<AudioDeviceInfo>,

        /// The index of the preferred/best default device for this backend.
        ///
        /// This will be `None` if the preferred device is not known at this time.
        default_i: Option<usize>,
    },
}

impl AudioBackendStatus {
    /// The index of the preferred/default device.
    ///
    /// This will be `None` if the backend is not running or if the default
    /// device is not known.
    pub fn default_device_i(&self) -> Option<usize> {
        match self {
            AudioBackendStatus::RunningWithSystemWideDevice(_) => Some(0),
            AudioBackendStatus::Running { default_i, .. } => *default_i,
            _ => None,
        }
    }

    /// Get the info of a particular audio device.
    ///
    /// This will be `None` if the backend is not running or if the given
    /// index is out of bounds.
    pub fn device_info(&self, index: usize) -> Option<&AudioDeviceInfo> {
        match self {
            // Always return the single "system-wide" device regardless of index.
            AudioBackendStatus::RunningWithSystemWideDevice(device_info) => Some(device_info),
            // Get the device info at the requested index.
            AudioBackendStatus::Running { devices, .. } => devices.get(index),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Hash)]
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
            AudioBackend::Alsa => true,

            #[cfg(feature = "pulseaudio")]
            AudioBackend::Pulseaudio => true,
            */
            AudioBackend::CoreAudio => false,

            /*
            #[cfg(feature = "jack-macos")]
            AudioBackend::JackMacOS => false,
            */
            AudioBackend::Wasapi => true,

            #[cfg(feature = "asio")]
            AudioBackend::Asio => true,
            /*
            #[cfg(feature = "jack-windows")]
            AudioBackend::JackWindows => false,
            */
        }
    }

    /*
    /// If this is true, then it means that this backend supports creating
    /// virtual ports that can be connected later.
    pub fn supports_creating_virtual_ports(&self) -> bool {
        match self {
            AudioBackend::Pipewire => true, // I think?

            #[cfg(feature = "jack-linux")]
            AudioBackend::JackLinux => true,

            /*
            #[cfg(feature = "alsa")]
            AudioBackend::Alsa => false,

            #[cfg(feature = "pulseaudio")]
            AudioBackend::Pulseaudio => false,
            */
            AudioBackend::CoreAudio => false, // I think?

            /*
            #[cfg(feature = "jack-macos")]
            AudioBackend::JackMacOS => true,
            */
            AudioBackend::Wasapi => false,

            #[cfg(feature = "asio")]
            AudioBackend::Asio => false,
            /*
            #[cfg(feature = "jack-windows")]
            AudioBackend::JackWindows => true,
            */
        }
    }
    */

    pub fn as_str(&self) -> &'static str {
        match self {
            AudioBackend::Pipewire => "Pipewire",

            #[cfg(feature = "jack-linux")]
            AudioBackend::JackLinux => "Jack",

            /*
            #[cfg(feature = "alsa")]
            AudioBackend::Alsa => "Alsa",

            #[cfg(feature = "pulseaudio")]
            AudioBackend::Pulseaudio => "Pulseaudio",
            */
            AudioBackend::CoreAudio => "CoreAudio", // I think?

            /*
            #[cfg(feature = "jack-macos")]
            AudioBackend::JackMacOS => "Jack",
            */
            AudioBackend::Wasapi => "WASAPI",

            #[cfg(feature = "asio")]
            Backend::Asio => "ASIO",
            /*
            #[cfg(feature = "jack-windows")]
            AudioBackend::JackWindows => "JACK",
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

impl MidiBackend {
    pub fn as_str(&self) -> &'static str {
        match self {
            MidiBackend::Pipewire => "Pipewire",

            #[cfg(feature = "jack-linux")]
            MidiBackend::JackLinux => "Jack",

            /*
            #[cfg(feature = "alsa")]
            MidiBackend::Alsa => "Alsa",

            #[cfg(feature = "pulseaudio")]
            MidiBackend::Pulseaudio => "Pulseaudio",
            */
            MidiBackend::CoreAudio => "CoreAudio", // I think?

            /*
            #[cfg(feature = "jack-macos")]
            Backend::JackMacOS => "Jack",
            */
            MidiBackend::Wasapi => "WASAPI",

            #[cfg(feature = "asio")]
            MidiBackend::Asio => "ASIO",
            /*
            #[cfg(feature = "jack-windows")]
            MidiBackend::JackWindows => "JACK",
            */
        }
    }
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
    pub sample_rates: SampleRateInfo,

    /// Information on the buffer/block size options.
    ///
    /// This is irrelevant for ASIO devices because the buffer size is configured
    /// through the configuration GUI application for that device.
    pub buffer_sizes: AudioBufferSizeInfo,

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

/// The range of possible buffer/block sizes for an audio device.
#[derive(Debug, Clone)]
pub enum AudioBufferSizeInfo {
    /// The buffer/block size cannot be known at this time, and most likely
    /// won't be a fixed size when run.
    Unknown,

    /// The buffer/block size cannot be configured to a specific fixed size,
    /// and instead will alternate somewhere between these two bounds when
    /// run (if these bounds are even known).
    UnconfigurableNotFixed {
        /// The minimum buffer/block size (inclusive).
        ///
        /// This will be "unkown" if the minumum size is not known.
        min: String,
        /// The maximum buffer/block size (inclusive).
        ///
        /// This will be "unkown" if the maximum size is not known.
        max: String,
    },

    /// The buffer/block size cannot be configured to a specific fixed size,
    /// but instead will use the single given fixed size.
    UnconfigurableFixed(u32),

    /// A set list of available fixed buffer/block sizes
    FixedList {
        /// The list of available fixed buffer/block sizes
        options: Vec<u32>,

        /// The ***index*** of the default buffer/block size
        default_i: usize,
    },
}

/// The range of possible sample rates for an audio device.
#[derive(Debug, Clone)]
pub enum SampleRateInfo {
    /// The sample rate cannot be known at this time.
    Unknown,

    /// The sample rate cannot be configured to a specific value, but
    /// instead will use the single given value.
    Unconfigurable(u32),

    /// A set list of available sample rates
    List {
        /// The list of available sample rates
        options: Vec<u32>,

        /// The ***index*** of the default sample rate
        default_i: usize,
    },
}

/// The layout of audio channels
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChannelLayout {
    /// The device has no ports
    Empty,

    /// The device has not specified the default channel layout of its ports.
    Unspecified,

    /// Single channel
    Mono,

    /// [left, right]
    Stereo,
    // TODO: More channel layouts
}

/// The default layout of audio channels
#[derive(Debug, Clone)]
pub struct DefaultChannelLayout {
    /// The layout of audio channels
    pub layout: ChannelLayout,

    /// The index of each device port for each channel, in order.
    pub device_ports: Vec<usize>,
}

impl DefaultChannelLayout {
    pub fn empty() -> Self {
        Self { layout: ChannelLayout::Empty, device_ports: Vec::new() }
    }
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

    /// The running status of this backend, along with all the available devices
    /// and their configurations.
    pub status: MidiBackendStatus,
}

/// The running status of this backend, along with all the available devices
/// and their configurations.
#[derive(Debug, Clone)]
pub enum MidiBackendStatus {
    /// The midi backend is not running/or is not available.
    NotRunning,

    /// The midi backend is running, but no devices were found.
    RunningButNoDevices,

    /// The midi backend is running with a list of available devices to choose from.
    Running {
        /// The input devices that are available in this backend.
        in_devices: Vec<MidiDeviceInfo>,

        /// The output devices that are available in this backend.
        out_devices: Vec<MidiDeviceInfo>,

        /// The index of the preferred/best default input device for this backend.
        default_in_i: Option<usize>,

        /// The index of the preferred/best default output device for this backend.
        default_out_i: Option<usize>,
    },
}

#[cfg(feature = "midi")]
/// Information about a particular midi device, including all its available
/// configurations.
#[derive(Debug, Clone)]
pub struct MidiDeviceInfo {
    pub id: DeviceID,
    // TODO: More information about the MIDI device
}
