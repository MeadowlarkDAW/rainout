use crate::{AudioBackend, DeviceID};

#[cfg(feature = "midi")]
use crate::MidiBackend;

/// A full configuration of audio and midi devices to connect to.
#[derive(Debug, Clone)]
pub struct Config {
    /// The type of the audio backend to use.
    pub audio_backend: AudioBackend,

    /// The ID of the audio device to use.
    pub audio_device: DeviceID,

    /// The names of the audio input ports to use.
    ///
    /// The buffers presented in the `ProcessInfo::audio_inputs` will appear in this exact same
    /// order.
    pub audio_in_ports: Vec<String>,

    /// The names of the audio output ports to use.
    ///
    /// The buffers presented in the `ProcessInfo::audio_outputs` will appear in this exact same
    /// order.
    pub audio_out_ports: Vec<String>,

    /// The sample rate to use.
    pub sample_rate: u32,

    /// The buffer size configuration for this device.
    pub buffer_size: AudioBufferSizeConfig,

    #[cfg(feature = "midi")]
    /// The configuration for MIDI devices.
    ///
    /// Set this to `None` to use no MIDI devices in the stream.
    pub midi_config: Option<MidiConfig>,
}

/// The buffer size configuration for an audio device.
#[derive(Debug, Clone, Copy)]
pub struct AudioBufferSizeConfig {
    /// If `Some`, then the backend will attempt to use a fixed size buffer of the
    /// given size. If this is `None`, then the backend will attempt to use the default
    /// fixed buffer size (if there is one).
    pub try_fixed_buffer_size: Option<u32>,

    /// If the backend fails to set a fixed buffer size from `try_fixed_buffer_size`,
    /// then unfixed buffer sizes will be used instead. This number will be the
    /// maximum size of a buffer that will be passed into the `process()` method in
    /// that case.
    pub fallback_max_buffer_size: u32,
}

#[cfg(feature = "midi")]
/// A full configuration of midi devices to connect to.
#[derive(Debug, Clone)]
pub struct MidiConfig {
    /// The type of the audio backend to use.
    pub backend: MidiBackend,

    /// The IDs of the input MIDI devices to use.
    ///
    /// The buffers presented in the `ProcessInfo::midi_inputs` will appear in this exact same
    /// order.
    pub in_devices: Vec<DeviceID>,

    /// The IDs of the output MIDI devices to use.
    ///
    /// The buffers presented in the `ProcessInfo::midi_outputs` will appear in this exact
    /// same order.
    pub out_devices: Vec<DeviceID>,
}
