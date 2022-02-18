use crate::error::{
    ChangeAudioBufferSizeError, ChangeAudioPortConfigError, FatalStreamError, RunConfigError,
    StreamError,
};
use crate::error_behavior::ErrorBehavior;
use crate::{platform, AudioBufferSizeConfig, Config, DeviceID, ProcessInfo, StreamInfo};

#[cfg(feature = "midi")]
use crate::error::ChangeMidiDeviceConfigError;

/// Get the estimated total latency of a particular configuration before running it.
///
/// `None` will be returned if the latency is not known at this time or if the
/// given config is invalid.
pub fn estimated_latency(config: &Config) -> Option<u32> {
    platform::estimated_latency(config)
}

/// Get the sample rate of a particular configuration before running it.
///
/// `None` will be returned if the sample rate is not known at this time or if the
/// given config is invalid.
pub fn sample_rate(config: &Config) -> Option<u32> {
    platform::sample_rate(config)
}

/// A processor for a stream.
pub trait ProcessHandler: 'static + Send {
    /// Initialize/allocate any buffers here. This will only be called once on
    /// creation.
    fn init(&mut self, stream_info: &StreamInfo);

    /// This gets called if the user made a change to the configuration that does not
    /// require restarting the audio thread.
    fn stream_changed(&mut self, stream_info: &StreamInfo);

    /// Process the current buffers. This will always be called on a realtime thread.
    fn process<'a>(&mut self, proc_info: ProcessInfo<'a>);
}

/// An error handler for a stream.
pub trait ErrorHandler: 'static + Send + Sync {
    /// Called when a non-fatal error occurs (any error that does not require the audio
    /// thread to restart).
    fn nonfatal_error(&mut self, error: StreamError);

    /// Called when a fatal error occurs (any error that requires the audio thread to
    /// restart).
    fn fatal_error(self, error: FatalStreamError);
}

#[derive(Debug, Clone)]
pub struct RunOptions {
    /// If `Some`, then the backend will use this name as the
    /// client name that appears in the audio server. This is only relevent for some
    /// backends like Jack.
    ///
    /// By default this is set to `None`.
    pub use_application_name: Option<String>,

    #[cfg(feature = "midi")]
    /// The maximum number of events a MIDI buffer can hold.
    ///
    /// By default this is set to `1024`.
    pub midi_buffer_size: u32,

    /// If true, then the backend will mark every input audio buffer that is
    /// silent (all `0.0`s) before each call to `process()`.
    ///
    /// If false, then the backend won't do this check and every buffer will
    /// be marked as not silent.
    ///
    /// By default this is set to `false`.
    pub check_for_silent_inputs: bool,

    /// How the system should respond to various errors.
    pub error_behavior: ErrorBehavior,
}

impl Default for RunOptions {
    fn default() -> Self {
        Self {
            use_application_name: None,

            #[cfg(feature = "midi")]
            midi_buffer_size: 1024,

            check_for_silent_inputs: false,
            error_behavior: ErrorBehavior::default(),
        }
    }
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
pub fn run<P: ProcessHandler, E: ErrorHandler>(
    config: &Config,
    options: &RunOptions,
    process_handler: P,
    error_handler: E,
) -> Result<StreamHandle<P, E>, RunConfigError> {
    platform::run(config, options, process_handler, error_handler)
}

/// The handle to a running audio/midi stream.
///
// When this gets dropped, the stream (audio thread) will automatically stop. This
/// is the intended method for stopping a stream.
pub struct StreamHandle<P: ProcessHandler, E: ErrorHandler> {
    pub(crate) platform_handle: Box<dyn PlatformStreamHandle<P, E>>,
}

impl<P: ProcessHandler, E: ErrorHandler> StreamHandle<P, E> {
    /// Returns the actual configuration of the running stream. This may differ
    /// from the configuration passed into the `run()` method.
    pub fn stream_info(&self) -> &StreamInfo {
        self.platform_handle.stream_info()
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
        self.platform_handle.change_audio_port_config(audio_in_ports, audio_out_ports)
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
        self.platform_handle.change_audio_buffer_size_config(config)
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
        self.platform_handle.change_midi_device_config(in_devices, out_devices)
    }

    // It may be possible to also add `change_sample_rate_config()` here, but
    // I'm not sure how useful this would actually be.

    /// Returns whether or not this backend supports changing the audio bus
    /// configuration while the audio thread is running.
    pub fn can_change_audio_port_config(&self) -> bool {
        self.platform_handle.can_change_audio_port_config()
    }

    // Returns whether or not this backend supports changing the buffer size
    // configuration while the audio thread is running.
    pub fn can_change_audio_buffer_size_config(&self) -> bool {
        self.platform_handle.can_change_audio_buffer_size_config()
    }

    #[cfg(feature = "midi")]
    /// Returns whether or not this backend supports changing the midi device
    /// config while the audio thread is running.
    pub fn can_change_midi_device_config(&self) -> bool {
        self.platform_handle.can_change_midi_device_config()
    }
}

pub(crate) trait PlatformStreamHandle<P: ProcessHandler, E: ErrorHandler> {
    /// Returns the actual configuration of the running stream. This may differ
    /// from the configuration passed into the `run()` method.
    fn stream_info(&self) -> &StreamInfo;

    /// Change the audio port configuration while the audio thread is still running.
    /// Support for this will depend on the backend.
    ///
    /// If the given config is invalid, an error will be returned with no
    /// effect on the running audio thread.
    fn change_audio_port_config(
        &mut self,
        audio_in_ports: Option<Vec<String>>,
        audio_out_ports: Option<Vec<String>>,
    ) -> Result<(), ChangeAudioPortConfigError>;

    /// Change the buffer size configuration while the audio thread is still running.
    /// Support for this will depend on the backend.
    ///
    /// If the given config is invalid, an error will be returned with no
    /// effect on the running audio thread.
    fn change_audio_buffer_size_config(
        &mut self,
        config: AudioBufferSizeConfig,
    ) -> Result<(), ChangeAudioBufferSizeError>;

    #[cfg(feature = "midi")]
    /// Change the midi device configuration while the audio thread is still running.
    /// Support for this will depend on the backend.
    ///
    /// If the given config is invalid, an error will be returned with no
    /// effect on the running audio thread.
    fn change_midi_device_config(
        &mut self,
        in_devices: Vec<DeviceID>,
        out_devices: Vec<DeviceID>,
    ) -> Result<(), ChangeMidiDeviceConfigError>;

    // It may be possible to also add `change_sample_rate_config()` here, but
    // I'm not sure how useful this would actually be.

    /// Returns whether or not this backend supports changing the audio bus
    /// configuration while the audio thread is running.
    fn can_change_audio_port_config(&self) -> bool;

    // Returns whether or not this backend supports changing the buffer size
    // configuration while the audio thread is running.
    fn can_change_audio_buffer_size_config(&self) -> bool;

    #[cfg(feature = "midi")]
    /// Returns whether or not this backend supports changing the midi device
    /// config while the audio thread is running.
    fn can_change_midi_device_config(&self) -> bool;
}
