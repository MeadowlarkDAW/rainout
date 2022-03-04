use crate::DeviceID;

#[cfg(feature = "midi")]
use crate::MidiControlScheme;

/// Information about a running stream.
#[derive(Debug, Clone)]
pub struct StreamInfo {
    /// The name of the audio backend.
    pub audio_backend: String,

    /// The version of the audio backend (if there is one available)
    ///
    /// (i.e. "1.2.10")
    pub audio_backend_version: Option<String>,

    /// The name/id of the audio device.
    pub audio_device: AudioDeviceStreamInfo,

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

    /// The total estimated latency of this stream in frames (if it is available)
    pub estimated_latency: Option<u32>,

    /// If this is `true`, then it means that the backend is checking
    /// each audio input buffer for silence before each call to the
    /// `process()` loop and marking the flag in `ProcessInfo`.
    pub checking_for_silent_inputs: bool,

    /// The information about the MIDI stream.
    ///
    /// If no MIDI stream is running, this will be `None`.
    #[cfg(feature = "midi")]
    pub midi_info: Option<MidiStreamInfo>,
}

#[derive(Debug, Clone)]
pub enum AudioDeviceStreamInfo {
    Single(DeviceID),
    LinkedInOut { input: Option<DeviceID>, output: Option<DeviceID> },
}

#[derive(Debug, Clone)]
pub struct StreamAudioPortInfo {
    /// The index of this audio port.
    pub port_index: usize,

    /// If the system port was found and is working correctly, this will
    /// be `true`. Otherwise if the system port was not found or it is not
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

impl StreamAudioBufferSize {
    pub fn max_buffer_size(&self) -> u32 {
        match self {
            Self::FixedSized(s) => *s,
            Self::UnfixedWithMaxSize(s) => *s,
        }
    }
}

#[cfg(feature = "midi")]
/// MIDI information about a running stream.
#[derive(Debug, Clone)]
pub struct MidiStreamInfo {
    /// The name of the midi backend.
    pub midi_backend: String,

    /// The names & status of the MIDI input devices.
    ///
    /// The buffers presented in the `ProcessInfo::midi_inputs` will
    /// appear in this exact same order.
    pub in_device_ports: Vec<StreamMidiPortInfo>,

    /// The names & status of the MIDI output devices.
    ///
    /// The buffers presented in the `ProcessInfo::midi_outputs` will
    /// appear in this exact same order.
    pub out_device_ports: Vec<StreamMidiPortInfo>,

    /// The allocated size for each MIDI buffer.
    pub midi_buffer_size: usize,
}

#[cfg(feature = "midi")]
#[derive(Debug, Clone)]
pub struct StreamMidiPortInfo {
    /// The name/ID of this device
    pub id: DeviceID,

    /// The index of this port for this device
    pub port_index: usize,

    /// The control scheme being used.
    pub control_scheme: MidiControlScheme,

    /// If the system device was found and is working correctly, this will
    /// be true. Otherwise if the system device was not found or it is not
    /// working correctly this will be false.
    ///
    /// Note even if this is `false`, the MIDI buffer for that port will
    /// still be passed to `ProcessInfo`. It will just be an empty buffer
    /// that won't do anything.
    pub success: bool,
}
