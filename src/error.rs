use std::error::Error;
use std::fmt;

use crate::{Backend, DeviceID};

#[cfg(feature = "midi")]
use crate::MAX_MIDI_MSG_SIZE;

#[derive(Debug, Clone)]
/// An error that caused the stream to stop.
pub enum StreamError {
    AudioServerShutdown { msg: Option<String> },
    AudioServerChangedSamplerate(u32),
    PlatformSpecific(Box<dyn Error>),
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
            StreamError::PlatformSpecific(e) => {
                write!(f, "Fatal stream error: {}", e)
            }
        }
    }
}

#[derive(Debug)]
pub enum RunConfigError {
    MalformedConfig(String),

    AudioBackendNotFound(Backend),
    AudioBackendNotInstalled(Backend),
    AudioBackendNotRunning(Backend),
    AudioDeviceNotFound(DeviceID),
    CouldNotUseSampleRate(u32),
    CouldNotUseBlockSize(u32),
    ConfigHasNoStereoOutput,
    AutoNoStereoOutputFound,

    #[cfg(any(feature = "jack-linux", feature = "jack-macos", feature = "jack-windows"))]
    JackAudioPortNotFound(String),
    #[cfg(any(feature = "jack-linux", feature = "jack-macos", feature = "jack-windows"))]
    JackNotEnabledForPlatform,

    #[cfg(feature = "midi")]
    MidiBackendNotFound(Backend),
    #[cfg(feature = "midi")]
    MidiDeviceNotFound(DeviceID),

    PlatformSpecific(Box<dyn Error>),
}
impl Error for RunConfigError {}
impl fmt::Display for RunConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RunConfigError::MalformedConfig(msg) => {
                write!(f, "Failed to run config: Malformed config: {}", msg)
            }
            RunConfigError::AudioBackendNotFound(b) => {
                write!(f, "Failed to run config: The audio backend {:?} was not found", b)
            }
            RunConfigError::AudioBackendNotInstalled(b) => {
                write!(
                    f,
                    "Failed to run config: The audio backend {:?} is not installed on the system",
                    b
                )
            }
            RunConfigError::AudioBackendNotRunning(b) => {
                write!(
                    f,
                    "Failed to run config: The audio backend {:?} is not running on the system",
                    b
                )
            }
            RunConfigError::AudioDeviceNotFound(a) => {
                write!(f, "Failed to run config: The audio device {} was not found", a)
            }
            RunConfigError::CouldNotUseSampleRate(s) => {
                write!(f, "Failed to run config: Could not use the sample rate {}", s)
            }
            RunConfigError::CouldNotUseBlockSize(b) => {
                write!(f, "Failed to run config: Could not use the block/buffer size {}", b)
            }
            RunConfigError::ConfigHasNoStereoOutput => {
                write!(f, "Failed to run config: Config must have at-least 2 audio output ports")
            }
            RunConfigError::AutoNoStereoOutputFound => {
                write!(f, "Failed to run config: Could not find an audio device with at-least 2 output ports")
            }

            RunConfigError::CouldNotUseExclusive => {
                write!(f, "Failed to run config: Could not run audio device in exclusive mode")
            }

            #[cfg(any(feature = "jack-linux", feature = "jack-macos", feature = "jack-windows"))]
            RunConfigError::JackAudioPortNotFound(p) => {
                write!(f, "Failed to run config: The Jack audio port {} was not found", p)
            }
            #[cfg(any(feature = "jack-linux", feature = "jack-macos", feature = "jack-windows"))]
            RunConfigError::JackNotEnabledForPlatform => {
                write!(f, "Failed to run config: Jack on this platform is not enabled by this application")
            }

            #[cfg(feature = "midi")]
            RunConfigError::MidiBackendNotFound(b) => {
                write!(f, "Failed to run config: The MIDI backend {:?} was not found", b)
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
pub enum ChangeBlockSizeError {
    NotSupportedByBackend, // TODO: more errors?
}
impl Error for ChangeBlockSizeError {}
impl fmt::Display for ChangeBlockSizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChangeBlockSizeError::NotSupportedByBackend => {
                write!(f, "Failed to change buffer size config: Not supported on this backend")
            }
        }
    }
}

#[cfg(feature = "midi")]
#[derive(Debug, Clone)]
pub enum ChangeMidiPortsError {
    NotSupportedByBackend, // TODO: more errors?
}
#[cfg(feature = "midi")]
impl Error for ChangeMidiPortsError {}
#[cfg(feature = "midi")]
impl fmt::Display for ChangeMidiPortsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChangeMidiPortsError::NotSupportedByBackend => {
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
