use crate::{Backend, DeviceID};

#[cfg(feature = "midi")]
use crate::MidiControlScheme;

/// Information about a running stream.
#[derive(Debug, Clone)]
pub struct StreamInfo {
    /// The audio backend
    pub audio_backend: Backend,

    /// The version of the audio backend (if there is one available)
    ///
    /// (i.e. "1.2.10")
    pub audio_backend_version: Option<String>,

    /// The name/id of the audio device.
    pub audio_device: AudioDeviceStreamInfo,

    /// The sample rate of the stream.
    pub sample_rate: u32,

    /// The audio buffer size.
    pub buffer_size: AudioBufferStreamInfo,

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
    /// Using a single audio device. This device may be output only, input
    /// only, or (most commonly) duplex.
    Single {
        id: DeviceID,

        /// If this is `false` then it means the app failed to connect to
        /// the system device and is using "fake/virtual" empty buffers
        /// instead which will only input and output silence.
        connected_to_system: bool,
    },

    /// Using an input/output device pair. This is only supported on some
    /// backends.
    LinkedInOut {
        /// The name/ID of the input device.
        ///
        /// If no input device was given in the configuration then this will
        /// be `None`.
        input: Option<DeviceID>,

        /// The name/ID of the input device.
        ///
        /// If no output device was given in the configuration then this will
        /// be `None`.
        output: Option<DeviceID>,

        /// If this is `false` then it means the app failed to connect to
        /// the system input device and is using "fake/virtual" empty buffers
        /// instead which will only input silence.
        ///
        /// This is not relevant if no input device was given in the
        /// configuration (`input` is `None`).
        in_connected_to_system: bool,

        /// If this is `false` then it means the app failed to connect to
        /// the system input device and is using "fake/virtual" empty buffers
        /// instead which will only input silence.
        ///
        /// This is not relevant if no input device was given in the
        /// configuration (`input` is `None`).
        out_connected_to_system: bool,
    },

    #[cfg(any(feature = "jack-linux", feature = "jack-macos", feature = "jack-windows"))]
    /// Using the Jack system audio device.
    Jack {
        /// The names of the audio input ports, as well as whether or not
        /// this port is connected to a system port (`true`), or if it is a
        /// "virtual" port that is not connected to any system port (`false`).
        in_ports: Vec<(String, bool)>,

        /// The names of the audio output ports, as well as whether or not
        /// this port is connected to a system port (`true`), or if it is a
        /// "virtual" port that is not connected to any system port (`false`).
        out_ports: Vec<(String, bool)>,
    },
}

/// The audio buffer size of a stream.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AudioBufferStreamInfo {
    FixedSized(u32),
    UnfixedWithMaxSize(u32),
    UnfixedWithMinSize(u32),
    Unfixed,
}

impl AudioBufferStreamInfo {
    pub fn max_buffer_size(&self) -> Option<u32> {
        match self {
            Self::FixedSized(s) => Some(*s),
            Self::UnfixedWithMaxSize(s) => Some(*s),
            Self::UnfixedWithMinSize(_) => None,
            Self::Unfixed => None,
        }
    }
}

#[cfg(feature = "midi")]
/// MIDI information about a running stream.
#[derive(Debug, Clone)]
pub struct MidiStreamInfo {
    /// The midi backend
    pub midi_backend: Backend,

    /// The names & status of the MIDI input ports.
    ///
    /// The buffers presented in the `ProcessInfo::midi_ins` will
    /// appear in this exact same order.
    pub in_ports: Vec<MidiPortStreamInfo>,

    /// The names & status of the MIDI output ports.
    ///
    /// The buffers presented in the `ProcessInfo::midi_outs` will
    /// appear in this exact same order.
    pub out_ports: Vec<MidiPortStreamInfo>,

    /// The allocated size for each MIDI buffer.
    pub midi_buffer_size: usize,
}

#[cfg(feature = "midi")]
#[derive(Debug, Clone)]
pub struct MidiPortStreamInfo {
    /// The name/ID of this device
    pub id: DeviceID,

    /// The index of this port for this device
    pub port_index: usize,

    /// The control scheme being used.
    pub control_scheme: MidiControlScheme,

    /// If this is `false` then it means the app failed to connect to
    /// the port on the system MIDI device and is using "fake/virtual"
    /// empty buffers instead which will not do anything.
    pub connected_to_system: bool,
}
