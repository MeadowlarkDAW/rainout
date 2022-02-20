use std::error::Error;
use std::fmt;

use crate::AudioBackend;
#[cfg(feature = "midi")]
use crate::MAX_MIDI_MSG_SIZE;

#[derive(Debug, Clone)]
/// An error that caused the stream to stop.
pub enum StreamError {
    AudioServerShutdown { msg: Option<String> },
    AudioServerChangedSamplerate(u32),
    // TODO
}
impl Error for StreamError {}
impl fmt::Display for StreamError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StreamError::AudioServerShutdown { msg } => {
                if let Some(msg) = msg {
                    write!(f, "Fatal stream error: the audio server was shut down: {}", msg)
                } else {
                    write!(f, "Fatal stream error: the audio server was shut down")
                }
            }
            StreamError::AudioServerChangedSamplerate(sr) => {
                write!(f, "Fatal stream error: the audio server changed its sample rate to: {}", sr)
            }
        }
    }
}

#[derive(Debug)]
pub enum RunConfigError {
    AudioBackendNotFound(AudioBackend),
    AudioPortNotFound(String),
    #[cfg(feature = "midi")]
    MidiDeviceNotFound(String),
    PlatformSpecific(Box<dyn Error>),
}
impl Error for RunConfigError {}
impl fmt::Display for RunConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RunConfigError::AudioBackendNotFound(b) => {
                write!(f, "Failed to run config: The audio backend {:?} was not found", b)
            }
            RunConfigError::AudioPortNotFound(p) => {
                write!(f, "Failed to run config: The audio port {} was not found", p)
            }
            #[cfg(feature = "midi")]
            RunConfigError::MidiDeviceNotFound(m) => {
                write!(f, "Failed to run config: The midi device {} was not found", m)
            }
            RunConfigError::PlatformSpecific(e) => {
                write!(f, "Failed to run config: {}", e)
            }
        }
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
