mod wasapi;

use crate::error::RunConfigError;
use crate::{
    AudioBackendOptions, AudioDeviceConfigOptions, DeviceID, ProcessHandler, RainoutConfig,
    RunOptions, StreamHandle,
};

#[cfg(feature = "midi")]
use crate::MidiBackendOptions;

#[cfg(feature = "asio")]
use crate::AsioAudioDeviceOptions;

#[cfg(feature = "jack-windows")]
use crate::{error::JackEnumerationError, JackAudioDeviceOptions};

/// Returns the list available audio backends for this platform.
///
/// These are ordered with the first item (index 0) being the most highly
/// preferred default backend.
pub fn available_audio_backends() -> &'static [&'static str] {
    &["wasapi"]
}

#[cfg(feature = "midi")]
/// Returns the list available midi backends for this platform.
///
/// These are ordered with the first item (index 0) being the most highly
/// preferred default backend.
pub fn available_midi_backends() -> &'static [&'static str] {
    &[]
}

/// Returns the list of available audio devices for the given backend.
///
/// This will return an error if the backend with the given name could
/// not be found.
pub fn enumerate_audio_backend(backend: &str) -> Result<AudioBackendOptions, ()> {
    match backend {
        "wasapi" => wasapi::enumerate(),
        _ => Err(()),
    }
}

/// Returns the configuration options for the given device.
///
/// This will return an error if the backend or the device could not
/// be found.
pub fn enumerate_audio_device(
    backend: &str,
    device: &DeviceID,
) -> Result<AudioDeviceConfigOptions, ()> {
    todo!()
}

#[cfg(feature = "asio")]
/// Returns the configuration options for the given ASIO device.
///
/// This will return an error if the device could not be found.
pub fn enumerate_asio_audio_device(device: &DeviceID) -> Result<AsioAudioDeviceOptions, ()> {
    todo!()
}

#[cfg(feature = "jack-windows")]
/// Returns the configuration options for "monolithic" system-wide Jack
/// audio device.
///
/// This will return an error if Jack is not installed on the system
/// or if the Jack server is not running.
pub fn enumerate_jack_audio_device() -> Result<JackAudioDeviceOptions, JackEnumerationError> {
    todo!()
}

#[cfg(feature = "midi")]
/// Returns the list of available midi devices for the given backend.
///
/// This will return an error if the backend with the given name could
/// not be found.
pub fn enumerate_midi_backend(backend: &str) -> Result<MidiBackendOptions, ()> {
    todo!()
}

/// Get the estimated total latency of a particular configuration before running it.
///
/// `None` will be returned if the latency is not known at this time or if the
/// given config is invalid.
pub fn estimated_latency(config: &RainoutConfig) -> Option<u32> {
    todo!()
}

/// Get the sample rate of a particular configuration before running it.
///
/// `None` will be returned if the sample rate is not known at this time or if the
/// given config is invalid.
pub fn sample_rate(config: &RainoutConfig) -> Option<u32> {
    todo!()
}

/// Run the given configuration in an audio thread.
///
/// * `config`: The configuration to use.
/// * `options`: Various options for the stream.
/// * `process_handler`: An instance of your process handler.
/// * `error_handler`: An instance of your error handler.
///
/// If an error is returned, then it means the config failed to run and no audio
/// thread was spawned.
pub fn run<P: ProcessHandler>(
    config: &RainoutConfig,
    options: &RunOptions,
    process_handler: P,
) -> Result<StreamHandle<P>, RunConfigError> {
    todo!()
}
