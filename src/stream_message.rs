use crate::{error::StreamError, DeviceID};

#[non_exhaustive]
#[derive(Debug, Clone)]
/// A message sent from the audio thread.
pub enum StreamMsg {
    /// An audio device was unplugged while the stream was running. Any connected
    /// ports will input/output silence.
    AudioDeviceDisconnected(DeviceID),

    /// An audio device was reconnected while the stream was running. Any connected
    /// ports will function properly now.
    ///
    /// This will only be sent after an `AudioDeviceDisconnected` event.
    AudioDeviceReconnected(DeviceID),

    #[cfg(feature = "midi")]
    /// The MIDI output device was not found. This port will produce no MIDI events.
    MidiDeviceDisconnected(DeviceID),

    #[cfg(feature = "midi")]
    /// A MIDI device was reconnected while the stream was running. Any connected
    /// ports will function properly now.
    ///
    /// This will only be sent after an `MidiDeviceDisconnected` event.
    MidiDeviceReconnected(DeviceID),

    /// An error that caused the stream to close. Please discard this Stream Handle
    /// channel and prepare to start a new stream.
    Error(StreamError),

    /// The audio stream was closed gracefully. Please discard this Stream Handle.
    Closed,
}
