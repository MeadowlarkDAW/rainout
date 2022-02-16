use std::error::Error;
use std::fmt;

#[cfg(feature = "midi")]
use crate::MAX_MIDI_MSG_SIZE;

#[derive(Debug, Clone)]
pub enum StreamError {
    // TODO
}
impl Error for StreamError {}
impl fmt::Display for StreamError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub enum FatalStreamError {
    // TODO
}
impl Error for FatalStreamError {}
impl fmt::Display for FatalStreamError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub enum RunConfigError {
    // TODO
}
impl Error for RunConfigError {}
impl fmt::Display for RunConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub enum ChangeAudioPortConfigError {
    // TODO
}
impl Error for ChangeAudioPortConfigError {}
impl fmt::Display for ChangeAudioPortConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub enum ChangeAudioBufferSizeError {
    // TODO
}
impl Error for ChangeAudioBufferSizeError {}
impl fmt::Display for ChangeAudioBufferSizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub enum ChangeMidiDeviceConfigError {
    // TODO
}
impl Error for ChangeMidiDeviceConfigError {}
impl fmt::Display for ChangeMidiDeviceConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

#[cfg(feature = "midi")]
#[derive(Debug)]
pub enum MidiBufferPushError {
    /// The buffer is full.
    BufferFull,

    /// The given midi event is too long.
    EventTooLong(usize),
}
#[cfg(feature = "midi")]
impl std::error::Error for MidiBufferPushError {}
#[cfg(feature = "midi")]
impl std::fmt::Display for MidiBufferPushError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MidiBufferPushError::BufferFull => {
                write!(f, "Buffer is full",)
            }
            MidiBufferPushError::EventTooLong(len) => {
                write!(
                    f,
                    "Event with length {} is longer than the maximum length {}",
                    len, MAX_MIDI_MSG_SIZE,
                )
            }
        }
    }
}
