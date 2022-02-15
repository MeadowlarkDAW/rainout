/// Information about a running stream.
#[derive(Debug, Clone)]
pub struct StreamInfo {
    /// The name of the audio backend.
    pub audio_backend: String,

    /// The version of the audio backend (if there is one available)
    ///
    /// (i.e. "1.2.10")
    pub audio_backend_version: Option<String>,

    /// The name of the audio device.
    pub audio_device: String,

    /// The audio input ports in this stream.
    ///
    /// The buffers presented in the `ProcessInfo::audio_inputs` will
    /// appear in this exact same order.
    pub audio_in_ports: Vec<StreamAudioPortInfo>,

    /// The audio output ports in this stream.
    ///
    /// The buffers presented in the `ProcessInfo::audio_outputs` will
    /// appear in this exact same order.
    pub audio_out_ports: Vec<StreamAudioPortInfo>,

    /// The sample rate of the stream.
    pub sample_rate: u32,

    /// The audio buffer size.
    pub buffer_size: StreamAudioBufferSize,

    /// The total latency of this stream in frames (if it is available)
    pub latency: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct StreamAudioPortInfo {
    /// The name of this audio port.
    pub name: String,

    /// If the system port was found and is working correctly, this will
    /// be true. Otherwise if the system port was not found or it is not
    /// working correctly this will be false.
    ///
    /// Note even if this is `false`, the buffer for that port will still
    /// be passed to `ProcessInfo`. It will just be filled with silence
    /// instead and not do anything.
    pub success: bool,
}

/// The audio buffer size of a stream.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StreamAudioBufferSize {
    FixedSized(u32),
    UnfixedWithMaxSize(u32),
}

/// MIDI information about a running stream.
#[derive(Debug, Clone)]
pub struct MidiStreamInfo {
    /// The name of the MIDI backend.
    pub audio_backend: String,

    /// The names of the MIDI input devices.
    ///
    /// The buffers presented in the `ProcessInfo::midi_inputs` will
    /// appear in this exact same order.
    pub in_devices: Vec<StreamMidiDeviceInfo>,

    /// The names of the MIDI output devices.
    ///
    /// The buffers presented in the `ProcessInfo::midi_outputs` will
    /// appear in this exact same order.
    pub out_devices: Vec<StreamMidiDeviceInfo>,
}

#[derive(Debug, Clone)]
pub struct StreamMidiDeviceInfo {
    /// The name of this MIDI device.
    pub name: String,

    /// If the system device was found and is working correctly, this will
    /// be true. Otherwise if the system device was not found or it is not
    /// working correctly this will be false.
    ///
    /// Note even if this is `false`, the MIDI buffer for that port will
    /// still be passed to `ProcessInfo`. It will just be an empty buffer
    /// that won't do anything.
    pub success: bool,
}
