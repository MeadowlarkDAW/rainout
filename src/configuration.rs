use std::fmt::Debug;

use crate::DeviceID;

#[cfg(feature = "serde-config")]
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
/// Specifies whether to use a specific configuration or to automatically
/// select the best configuration.
pub enum AutoOption<T: Debug + Clone + PartialEq> {
    /// Use this specific configuration.
    Use(T),

    /// Automatically select the best configuration.
    Auto,
}

#[cfg(not(feature = "serde-config"))]
#[derive(Debug, Clone, PartialEq)]
/// Specifies whether to use a specific configuration or to automatically
/// select the best configuration.
pub enum AutoOption<T: Debug + Clone + PartialEq> {
    /// Use this specific configuration.
    Use(T),

    /// Automatically select the best configuration.
    Auto,
}

impl<T: Debug + Clone + PartialEq> Default for AutoOption<T> {
    fn default() -> Self {
        AutoOption::Auto
    }
}

#[cfg(feature = "serde-config")]
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
/// The configuration of audio and MIDI backends and devices.
pub struct RustyDawIoConfig {
    /// The audio backend to use.
    ///
    /// Set this to `AutoOption::Auto` to automatically select the best
    /// backend to use.
    pub audio_backend: AutoOption<String>,

    /// The audio device/devices to use.
    ///
    /// Set this to `AudioDeviceConfig::Auto` to automatically select the best
    /// audio device to use.
    pub audio_device: AudioDeviceConfig,

    /// The sample rate to use.
    ///
    /// Set this to `AutoOption::Auto` to automatically select the best
    /// sample rate to use.
    pub sample_rate: AutoOption<u32>,

    /// The block/buffer size to use.
    ///
    /// Set this to `AutoOption::Auto` to automatically select the best
    /// buffer/block size to use.
    pub block_size: AutoOption<u32>,

    /// The indexes of the audio input ports to use.
    ///
    /// The buffers presented in `ProcInfo::audio_in` will appear in this
    /// exact same order.
    ///
    /// Set this to `AutoOption::Auto` to automatically select the best
    /// configuration of input ports to use.
    ///
    /// You may also pass in an empty Vec to have no audio inputs.
    ///
    /// This is not relevent when the audio backend is Jack.
    pub input_channels: AutoOption<Vec<usize>>,

    /// The indexes of the audio output ports to use.
    ///
    /// The buffers presented in `ProcInfo::audio_out` will appear in this
    /// exact same order.
    ///
    /// Set this to `AutoOption::Auto` to automatically select the best
    /// configuration of output ports to use.
    ///
    /// You may also pass in an empty Vec to have no audio outputs.
    ///
    /// This is not relevent when the audio backend is Jack.
    pub output_channels: AutoOption<Vec<usize>>,

    #[cfg(any(feature = "jack-linux", feature = "jack-macos", feature = "jack-windows"))]
    /// When the audio backend is Jack, the names of the audio input ports
    /// to use.
    ///
    /// The buffers presented in `ProcInfo::audio_in` will appear in this
    /// exact same order.
    ///
    /// If a port with the given name does not exist, then an unconnected
    /// virtual port with that same name will be created.
    ///
    /// Set this to `AutoOption::Auto` to automatically select the best
    /// configuration of input ports to use.
    ///
    /// You may also pass in an empty Vec to have no audio inputs.
    ///
    /// This is only relevent when the audio backend is Jack.
    pub jack_input_ports: AutoOption<Vec<String>>,

    #[cfg(any(feature = "jack-linux", feature = "jack-macos", feature = "jack-windows"))]
    /// When the audio backend is Jack, the names of the audio output ports
    /// to use.
    ///
    /// The buffers presented in `ProcInfo::audio_out` will appear in this
    /// exact same order.
    ///
    /// If a port with the given name does not exist, then an unconnected
    /// virtual port with that same name will be created.
    ///
    /// Set this to `AutoOption::Auto` to automatically select the best
    /// configuration of output ports to use.
    ///
    /// You may also pass in an empty Vec to have no audio outputs.
    ///
    /// This is only relevent when the audio backend is Jack.
    pub jack_output_ports: AutoOption<Vec<String>>,

    /// If `true` then it means that the application can request to take
    /// exclusive access of the device to improve latency.
    ///
    /// This is only relevant for WASAPI on Windows. This will always be
    /// `false` on other backends and platforms.
    pub take_exclusive_access: bool,

    #[cfg(feature = "midi")]
    /// The configuration of MIDI devices.
    ///
    /// Set this to `None` to use no MIDI devices.
    pub midi_config: Option<MidiConfig>,
}

#[cfg(not(feature = "serde-config"))]
#[derive(Debug, Clone, PartialEq)]
/// The configuration of audio and MIDI backends and devices.
pub struct RustyDawIoConfig {
    /// The audio backend to use.
    ///
    /// Set this to `AutoOption::Auto` to automatically select the best
    /// backend to use.
    pub audio_backend: AutoOption<String>,

    /// The audio device/devices to use.
    ///
    /// Set this to `AudioDeviceConfig::Auto` to automatically select the best
    /// audio device to use.
    pub audio_device: AudioDeviceConfig,

    /// The sample rate to use.
    ///
    /// Set this to `AutoOption::Auto` to automatically select the best
    /// sample rate to use.
    pub sample_rate: AutoOption<u32>,

    /// The block/buffer size to use.
    ///
    /// Set this to `AutoOption::Auto` to automatically select the best
    /// buffer/block size to use.
    pub block_size: AutoOption<u32>,

    /// The indexes of the audio input ports to use.
    ///
    /// The buffers presented in `ProcInfo::audio_in` will appear in this
    /// exact same order.
    ///
    /// Set this to `AutoOption::Auto` to automatically select the best
    /// configuration of input ports to use.
    ///
    /// You may also pass in an empty Vec to have no audio inputs.
    ///
    /// This is not relevent when the audio backend is Jack.
    pub input_channels: AutoOption<Vec<usize>>,

    /// The indexes of the audio output ports to use.
    ///
    /// The buffers presented in `ProcInfo::audio_out` will appear in this
    /// exact same order.
    ///
    /// Set this to `AutoOption::Auto` to automatically select the best
    /// configuration of output ports to use.
    ///
    /// You may also pass in an empty Vec to have no audio outputs.
    ///
    /// This is not relevent when the audio backend is Jack.
    pub output_channels: AutoOption<Vec<usize>>,

    #[cfg(any(feature = "jack-linux", feature = "jack-macos", feature = "jack-windows"))]
    /// When the audio backend is Jack, the names of the audio input ports
    /// to use.
    ///
    /// The buffers presented in `ProcInfo::audio_in` will appear in this
    /// exact same order.
    ///
    /// If a port with the given name does not exist, then an unconnected
    /// virtual port with that same name will be created.
    ///
    /// Set this to `AutoOption::Auto` to automatically select the best
    /// configuration of input ports to use.
    ///
    /// You may also pass in an empty Vec to have no audio inputs.
    ///
    /// This is only relevent when the audio backend is Jack.
    pub jack_input_ports: AutoOption<Vec<String>>,

    #[cfg(any(feature = "jack-linux", feature = "jack-macos", feature = "jack-windows"))]
    /// When the audio backend is Jack, the names of the audio output ports
    /// to use.
    ///
    /// The buffers presented in `ProcInfo::audio_out` will appear in this
    /// exact same order.
    ///
    /// If a port with the given name does not exist, then an unconnected
    /// virtual port with that same name will be created.
    ///
    /// Set this to `AutoOption::Auto` to automatically select the best
    /// configuration of output ports to use.
    ///
    /// You may also pass in an empty Vec to have no audio outputs.
    ///
    /// This is only relevent when the audio backend is Jack.
    pub jack_output_ports: AutoOption<Vec<String>>,

    /// If `true` then it means that the application can request to take
    /// exclusive access of the device to improve latency.
    ///
    /// This is only relevant for WASAPI on Windows. This will always be
    /// `false` on other backends and platforms.
    pub take_exclusive_access: bool,

    #[cfg(feature = "midi")]
    /// The configuration of MIDI devices.
    ///
    /// Set this to `None` to use no MIDI devices.
    pub midi_config: Option<MidiConfig>,
}

impl Default for RustyDawIoConfig {
    fn default() -> Self {
        RustyDawIoConfig {
            audio_backend: AutoOption::Auto,
            audio_device: AudioDeviceConfig::Auto,
            sample_rate: AutoOption::Auto,
            block_size: AutoOption::Auto,
            input_channels: AutoOption::Use(Vec::new()),
            output_channels: AutoOption::Auto,

            #[cfg(any(feature = "jack-linux", feature = "jack-macos", feature = "jack-windows"))]
            jack_input_ports: AutoOption::Use(Vec::new()),
            #[cfg(any(feature = "jack-linux", feature = "jack-macos", feature = "jack-windows"))]
            jack_output_ports: AutoOption::Auto,

            take_exclusive_access: false,

            #[cfg(feature = "midi")]
            midi_config: None,
        }
    }
}

#[cfg(feature = "serde-config")]
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
/// The configuration of which audio device/devices to use.
pub enum AudioDeviceConfig {
    /// Use a single audio device. These device may be output only, input
    /// only, or (most commonly) duplex.
    Single(DeviceID),

    /// Use an input/output device pair. This is only supported on some
    /// backends.
    LinkedInOut { input: Option<DeviceID>, output: Option<DeviceID> },

    /// Automatically select the best configuration.
    ///
    /// This should also be used when using the Jack backend.
    Auto,
}

#[cfg(not(feature = "serde-config"))]
#[derive(Debug, Clone, PartialEq)]
/// The configuration of which audio device/devices to use.
pub enum AudioDeviceConfig {
    /// Use a single audio device. These device may be output only, input
    /// only, or (most commonly) duplex.
    Single(DeviceID),

    /// Use an input/output device pair. This is only supported on some
    /// backends.
    LinkedInOut { input: DeviceID, output: DeviceID },

    /// Automatically select the best configuration.
    ///
    /// This should also be used when using the Jack backend.
    Auto,
}

impl Default for AudioDeviceConfig {
    fn default() -> Self {
        AudioDeviceConfig::Auto
    }
}

#[cfg(feature = "midi")]
#[cfg(feature = "serde-config")]
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
/// The configuration of the MIDI backend and devices.
pub struct MidiConfig {
    /// The MIDI backend to use.
    ///
    /// Set this to `AutoOption::Auto` to automatically select the best
    /// backend to use.
    pub midi_backend: AutoOption<String>,

    /// The names of the MIDI input ports to use.
    ///
    /// The buffers presented in `ProcInfo::midi_in` will appear in this
    /// exact same order.
    ///
    /// Set this to `AutoOption::Auto` to automatically select the best
    /// configuration of input ports to use.
    ///
    /// You may also pass in an empty Vec to have no MIDI inputs.
    pub input_ports: AutoOption<Vec<String>>,

    /// The names of the MIDI output ports to use.
    ///
    /// The buffers presented in `ProcInfo::midi_out` will appear in this
    /// exact same order.
    ///
    /// Set this to `AutoOption::Auto` to automatically select the best
    /// configuration of output ports to use.
    ///
    /// You may also pass in an empty Vec to have no MIDI outputs.
    pub output_ports: AutoOption<Vec<String>>,
}

#[cfg(feature = "midi")]
#[cfg(not(feature = "serde-config"))]
#[derive(Debug, Clone, PartialEq)]
/// The configuration of the MIDI backend and devices.
pub struct MidiConfig {
    /// The MIDI backend to use.
    ///
    /// Set this to `AutoOption::Auto` to automatically select the best
    /// backend to use.
    pub midi_backend: AutoOption<String>,

    /// The names of the MIDI input ports to use.
    ///
    /// The buffers presented in `ProcInfo::midi_in` will appear in this
    /// exact same order.
    ///
    /// Set this to `AutoOption::Auto` to automatically select the best
    /// configuration of input ports to use.
    ///
    /// You may also pass in an empty Vec to have no MIDI inputs.
    pub input_ports: AutoOption<Vec<String>>,

    /// The names of the MIDI output ports to use.
    ///
    /// The buffers presented in `ProcInfo::midi_out` will appear in this
    /// exact same order.
    ///
    /// Set this to `AutoOption::Auto` to automatically select the best
    /// configuration of output ports to use.
    ///
    /// You may also pass in an empty Vec to have no MIDI outputs.
    pub output_ports: AutoOption<Vec<String>>,
}

impl Default for MidiConfig {
    fn default() -> Self {
        MidiConfig {
            midi_backend: AutoOption::Auto,
            input_ports: AutoOption::Auto,
            output_ports: AutoOption::Use(Vec::new()),
        }
    }
}
