use std::error::Error;
use std::fmt;

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
    AudioBackendNotFound(String),
    AudioDeviceNotFound(String),
    AudioPortNotFound(String),
    CouldNotUseSampleRate(u32),
    CouldNotUseBlockSize(u32),

    #[cfg(feature = "midi")]
    MidiDeviceNotFound(String),

    PlatformSpecific(Box<dyn Error>),
}
impl Error for RunConfigError {}
impl fmt::Display for RunConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RunConfigError::AudioBackendNotFound(b) => {
                write!(f, "Failed to run config: The audio backend {} was not found", b)
            }
            RunConfigError::AudioDeviceNotFound(a) => {
                write!(f, "Failed to run config: The audio device {} was not found", a)
            }
            RunConfigError::AudioPortNotFound(p) => {
                write!(f, "Failed to run config: The audio port {} was not found", p)
            }
            RunConfigError::CouldNotUseSampleRate(s) => {
                write!(f, "Failed to run config: Could not use the sample rate {}", s)
            }
            RunConfigError::CouldNotUseBlockSize(b) => {
                write!(f, "Failed to run config: Could not use the block/buffer size {}", b)
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
    NotSupportedByBackend, // TODO: more errors?
}
impl Error for ChangeAudioPortConfigError {}
impl fmt::Display for ChangeAudioPortConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChangeAudioPortConfigError::NotSupportedByBackend => {
                write!(f, "Failed to change audio port config: Not supported on this backend")
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum ChangeAudioBufferSizeError {
    NotSupportedByBackend, // TODO: more errors?
}
impl Error for ChangeAudioBufferSizeError {}
impl fmt::Display for ChangeAudioBufferSizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChangeAudioBufferSizeError::NotSupportedByBackend => {
                write!(f, "Failed to change buffer size config: Not supported on this backend")
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum ChangeMidiDeviceConfigError {
    NotSupportedByBackend, // TODO: more errors?
}
impl Error for ChangeMidiDeviceConfigError {}
impl fmt::Display for ChangeMidiDeviceConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChangeMidiDeviceConfigError::NotSupportedByBackend => {
                write!(f, "Failed to change MIDI device config: Not supported on this backend")
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum JackEnumerationError {
    /// Jack is not installed on this system.
    NotInstalled,
    /// Jack server is not running.
    NotRunning,
    /// This application has not enabled Jack for this platform.
    NotEnabledForPlatform,
}
impl Error for JackEnumerationError {}
impl fmt::Display for JackEnumerationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JackEnumerationError::NotInstalled => write!(f, "Failed to enumerate backend: Jack is not installed on this system"),
            JackEnumerationError::NotRunning => write!(f, "Failed to enumerate backend: The Jack server is not running"),
            JackEnumerationError::NotEnabledForPlatform => write!(f, "Failed to enumerate backend: This application has not enabled Jack for this platform")
        }
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
