/// A full configuration of audio and midi devices to connect to.
#[derive(Debug, Clone)]
pub struct Config {
    /// The name of the audio backend to use.
    ///
    /// Set this to `None` to automatically select the default/best backend for the system.
    pub audio_backend: Option<String>,

    /// The name of the audio device to use.
    ///
    /// Set this to `None` to automatically select the default/best device for the backend.
    pub audio_device: Option<String>,

    /// The names of the audio input ports to use. The buffers presented in the `process()`
    /// thread will appear in this exact same order.
    ///
    /// Set this to `None` to automatically select the default input port layout for the device.
    pub audio_in_ports: Option<Vec<String>>,

    /// The names of the audio output ports to use. The buffers presented in the `process()`
    /// thread will appear in this exact same order.
    ///
    /// Set this to `None` to automatically select the default output port layout for the device.
    pub audio_out_ports: Option<Vec<String>>,

    /// The sample rate to use.
    ///
    /// Set this to `None` to use the default sample rate of the system device.
    pub sample_rate: Option<u32>,

    /// The buffer size configuration for this device.
    pub buffer_size: AudioBufferSizeConfig,

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

/// A full configuration of midi devices to connect to.
#[derive(Debug, Clone)]
pub struct MidiConfig {
    /// The name of the MIDI backend to use.
    ///
    /// Set this to `None` to automatically select the default/best backend for the system.
    pub backend: Option<String>,

    /// The names of the input MIDI devices to use. The buffers presented in the `process()`
    /// thread will appear in this exact same order.
    ///
    /// Set this to `None` to use the default input device for the backend.
    pub in_controllers: Option<Vec<String>>,

    /// The names of the output MIDI devices to use. The buffers presented in the `process()`
    /// thread will appear in this exact same order.
    ///
    /// Set this to `None` to use the default output device for the backend.
    pub out_controllers: Option<Vec<String>>,
}
