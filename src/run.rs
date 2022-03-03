use crate::error::{ChangeAudioBufferSizeError, ChangeAudioPortConfigError, RunConfigError};
use crate::error_behavior::ErrorBehavior;
use crate::{DeviceID, ProcessInfo, RustyDawIoConfig, StreamInfo};

#[cfg(feature = "midi")]
use crate::error::ChangeMidiDeviceConfigError;

/// Get the estimated total latency of a particular configuration before running it.
///
/// `None` will be returned if the latency is not known at this time or if the
/// given config is invalid.
pub fn estimated_latency(config: &RustyDawIoConfig) -> Option<u32> {
    todo!()
}

/// Get the sample rate of a particular configuration before running it.
///
/// `None` will be returned if the sample rate is not known at this time or if the
/// given config is invalid.
pub fn sample_rate(config: &RustyDawIoConfig) -> Option<u32> {
    todo!()
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

    /// The size of the audio thread to stream handle message buffer.
    ///
    /// By default this is set to `512`.
    pub msg_buffer_size: usize,
}

impl Default for RunOptions {
    fn default() -> Self {
        Self {
            use_application_name: None,

            #[cfg(feature = "midi")]
            midi_buffer_size: 1024,

            check_for_silent_inputs: false,
            error_behavior: ErrorBehavior::default(),
            msg_buffer_size: 512,
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
pub fn run<P: ProcessHandler>(
    config: &RustyDawIoConfig,
    options: &RunOptions,
    process_handler: P,
) -> Result<StreamHandle<P>, RunConfigError> {
    todo!()
}

/// The handle to a running audio/midi stream.
///
// When this gets dropped, the stream (audio thread) will automatically stop. This
/// is the intended method for stopping a stream.
pub struct StreamHandle<P: ProcessHandler> {
    /// The message channel that recieves notifications from the audio thread
    /// including any errors that have occurred.
    //pub messages: StreamMsgChannel,
    pub(crate) platform_handle: Box<dyn PlatformStreamHandle<P>>,
}

impl<P: ProcessHandler> StreamHandle<P> {
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
        buffer_size: u32,
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

pub(crate) trait PlatformStreamHandle<P: ProcessHandler> {
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
        buffer_size: u32,
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
