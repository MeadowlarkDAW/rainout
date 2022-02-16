use crate::error::{ChangeAudioBufferSizeError, ChangeAudioPortConfigError, RunConfigError};
use crate::{
    AudioBufferSizeConfig, Config, DeviceID, ErrorHandler, ProcessHandler, RunOptions,
    StreamHandle, StreamInfo,
};

#[cfg(feature = "midi")]
use crate::error::ChangeMidiDeviceConfigError;

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

pub fn run<P: ProcessHandler, E: ErrorHandler>(
    config: &Config,
    options: &RunOptions,
    process_handler: P,
    error_handler: E,
) -> Result<StreamHandle<P, E>, RunConfigError> {
    todo!()
}

pub struct PlatformStreamHandle<P: ProcessHandler, E: ErrorHandler> {
    process_handler: P,
    error_handler: E,
}

impl<P: ProcessHandler, E: ErrorHandler> PlatformStreamHandle<P, E> {
    /// Returns the actual configuration of the running stream. This may differ
    /// from the configuration passed into the `run()` method.
    pub fn stream_info(&self) -> &StreamInfo {
        todo!()
    }

    /// Change the audio port configuration while the audio thread is still running.
    /// Support for this will depend on the backend.
    ///
    /// If the given config is invalid, an error will be returned with no
    /// effect on the running audio thread.
    pub fn change_audio_port_config(
        &mut self,
        audio_in_ports: Option<Vec<String>>,
        audio_out_ports: Option<Vec<String>>,
    ) -> Result<(), ChangeAudioPortConfigError> {
        todo!()
    }

    /// Change the buffer size configuration while the audio thread is still running.
    /// Support for this will depend on the backend.
    ///
    /// If the given config is invalid, an error will be returned with no
    /// effect on the running audio thread.
    pub fn change_audio_buffer_size_config(
        &mut self,
        config: AudioBufferSizeConfig,
    ) -> Result<(), ChangeAudioBufferSizeError> {
        todo!()
    }

    #[cfg(feature = "midi")]
    /// Change the midi device configuration while the audio thread is still running.
    /// Support for this will depend on the backend.
    ///
    /// If the given config is invalid, an error will be returned with no
    /// effect on the running audio thread.
    pub fn change_midi_device_config(
        &mut self,
        in_devices: Vec<DeviceID>,
        out_devices: Vec<DeviceID>,
    ) -> Result<(), ChangeMidiDeviceConfigError> {
        todo!()
    }

    // It may be possible to also add `change_sample_rate_config()` here, but
    // I'm not sure how useful this would actually be.

    /// Returns whether or not this backend supports changing the audio bus
    /// configuration while the audio thread is running.
    pub fn can_change_audio_port_config(&self) -> bool {
        todo!()
    }

    // Returns whether or not this backend supports changing the buffer size
    // configuration while the audio thread is running.
    pub fn can_change_audio_buffer_size_config(&self) -> bool {
        todo!()
    }

    #[cfg(feature = "midi")]
    /// Returns whether or not this backend supports changing the midi device
    /// config while the audio thread is running.
    pub fn can_change_midi_device_config(&self) -> bool {
        todo!()
    }
}
