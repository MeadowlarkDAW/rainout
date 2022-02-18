use crate::{AudioBackend, AudioBackendInfo};

#[cfg(feature = "midi")]
use crate::{MidiBackend, MidiBackendInfo};

#[cfg(feature = "jack-linux")]
use super::jack_backend;

/// Returns the available audio backends for this platform.
pub fn available_audio_backends() -> &'static [AudioBackend] {
    &[
        #[cfg(feature = "jack-linux")]
        AudioBackend::JackLinux,
    ]
}

#[cfg(feature = "midi")]
/// Returns the available midi backends for this platform.
pub fn available_midi_backends() -> &'static [MidiBackend] {
    &[
        #[cfg(feature = "jack-linux")]
        MidiBackend::JackLinux,
    ]
}

/// Get information about a particular audio backend.
///
/// This will update the list of available devices as well as the the
/// status of whether or not this backend is running.
///
/// This will return an error if the backend is not available on this system.
pub fn enumerate_audio_backend(backend: AudioBackend) -> Result<AudioBackendInfo, ()> {
    match backend {
        #[cfg(feature = "jack-linux")]
        AudioBackend::JackLinux => Ok(jack_backend::enumerate_audio_backend()),
        _ => Err(()),
    }
}

#[cfg(feature = "midi")]
/// Get information about a particular midi backend.
///
/// This will update the list of available devices as well as the the
/// status of whether or not this backend is running.
///
/// This will return an error if the backend is not available on this system.
pub fn enumerate_midi_backend(backend: MidiBackend) -> Result<MidiBackendInfo, ()> {
    match backend {
        #[cfg(feature = "jack-linux")]
        MidiBackend::JackLinux => Ok(jack_backend::enumerate_midi_backend()),
        _ => Err(()),
    }
}
