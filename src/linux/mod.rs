use crate::error::RunConfigError;
use crate::{AudioBackend, AudioBackendInfo, Config, ProcessHandler, RunOptions, StreamHandle};

#[cfg(feature = "midi")]
use crate::{MidiBackend, MidiBackendInfo};

#[cfg(feature = "jack-linux")]
mod jack_backend;

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

/// Get the estimated total latency of a particular configuration before running it.
///
/// `None` will be returned if the latency is not known at this time or if the
/// given config is invalid.
pub fn estimated_latency(config: &Config) -> Option<u32> {
    todo!()
}

/// Get the sample rate of a particular configuration before running it.
///
/// `None` will be returned if the sample rate is not known at this time or if the
/// given config is invalid.
pub fn sample_rate(config: &Config) -> Option<u32> {
    todo!()
}

pub fn run<P: ProcessHandler>(
    config: &Config,
    options: &RunOptions,
    process_handler: P,
) -> Result<StreamHandle<P>, RunConfigError> {
    match config.audio_backend {
        #[cfg(feature = "jack-linux")]
        crate::AudioBackend::JackLinux => jack_backend::run(config, options, process_handler),
        b => Err(RunConfigError::AudioBackendNotFound(b)),
    }
}
