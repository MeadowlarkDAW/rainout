use crate::error::{ChangeAudioPortsError, ChangeBlockSizeError, RunConfigError};
use crate::{AutoOption, Backend, ProcessInfo, RainoutConfig, StreamInfo, StreamMsgChannel};

#[cfg(feature = "midi")]
use crate::{error::ChangeMidiPortsError, MidiPortConfig};

/// Get the estimated sample rate and total latency of a particular configuration
/// before running it.
///
/// `None` will be returned if the sample rate or latency is not known at this
/// time.
///
/// `(Option<SAMPLE_RATE>, Option<LATENCY>)`
pub fn estimated_sample_rate_and_latency(
    config: &RainoutConfig,
) -> Result<(Option<u32>, Option<u32>), RunConfigError> {
    let use_audio_backend = match config.audio_backend {
        AutoOption::Use(b) => b,
        AutoOption::Auto =>
        {
            #[cfg(all(target_os = "linux", feature = "jack-linux"))]
            Backend::Jack
        }
    };

    match use_audio_backend {
        Backend::Jack => {
            #[cfg(all(target_os = "linux", feature = "jack-linux"))]
            return crate::jack_backend::estimated_sample_rate_and_latency(config);
            #[cfg(all(target_os = "linux", not(feature = "jack-linux")))]
            {
                log::error!("The feature \"jack-linux\" is not enabled");
                return Err(RunConfigError::JackNotEnabledForPlatform);
            }

            #[cfg(all(target_os = "macos", feature = "jack-macos"))]
            return crate::jack_backend::estimated_sample_rate_and_latency(config);
            #[cfg(all(target_os = "macos", not(feature = "jack-macos")))]
            {
                log::error!("The feature \"jack-macos\" is not enabled");
                return Err(RunConfigError::JackNotEnabledForPlatform);
            }

            #[cfg(all(target_os = "windows", feature = "jack-windows"))]
            return crate::jack_backend::estimated_sample_rate_and_latency(config);
            #[cfg(all(target_os = "windows", not(feature = "jack-windows")))]
            {
                log::error!("The feature \"jack-windows\" is not enabled");
                return Err(RunConfigError::JackNotEnabledForPlatform);
            }
        }
        b => {
            log::error!("Unkown audio backend: {:?}", b);
            return Err(RunConfigError::AudioBackendNotFound(b));
        }
    }
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
/// Additional options for running a stream
pub struct RunOptions {
    /// If `Some`, then the backend will use this name as the
    /// client name that appears in the audio server. This is only relevent for some
    /// backends like Jack.
    ///
    /// By default this is set to `None`.
    pub use_application_name: Option<String>,

    /// If this is `true`, then the system will try to automatically connect to
    /// the default audio input port/ports when using `AutoOption::Auto`.
    ///
    /// If you only want audio outputs, then set this to `false`.
    ///
    /// By default this is set to `false`.
    pub auto_audio_inputs: bool,

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

    /// If `true`, then the system will return an error if it was not able to
    /// connect to a device with at-least two output ports. It will also try
    /// to avoid automatically connecting to devices with mono outputs.
    ///
    /// By default this is set to `true`.
    pub must_have_stereo_output: bool,

    /// If `true`, then the system will use empty (silent) buffers for any
    /// audio/MIDI ports that failed to connect instead of returning an
    /// error.
    ///
    /// By default this is set to `false`.
    pub empty_buffers_for_failed_ports: bool,

    /// The size of the audio thread to stream handle message buffer.
    ///
    /// By default this is set to `512`.
    pub msg_buffer_size: usize,
}

impl Default for RunOptions {
    fn default() -> Self {
        Self {
            use_application_name: None,

            auto_audio_inputs: false,

            #[cfg(feature = "midi")]
            midi_buffer_size: 1024,

            check_for_silent_inputs: false,
            must_have_stereo_output: true,
            empty_buffers_for_failed_ports: false,
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
    config: &RainoutConfig,
    options: &RunOptions,
    process_handler: P,
) -> Result<StreamHandle<P>, RunConfigError> {
    let use_audio_backend = match config.audio_backend {
        AutoOption::Use(b) => b,
        AutoOption::Auto =>
        {
            #[cfg(all(target_os = "linux", feature = "jack-linux"))]
            Backend::Jack
        }
    };

    let use_midi_backend = match &config.midi_config {
        Some(midi_config) => match midi_config.midi_backend {
            AutoOption::Use(b) => Some(b),
            AutoOption::Auto =>
            {
                #[cfg(all(target_os = "linux", feature = "jack-linux"))]
                Some(Backend::Jack)
            }
        },
        None => None,
    };

    let spawn_separate_midi_thread = if let Some(midi_backend) = use_midi_backend {
        midi_backend != use_audio_backend
    } else {
        false
    };

    if spawn_separate_midi_thread {
        todo!()
    } else {
        match use_audio_backend {
            Backend::Jack => {
                #[cfg(all(target_os = "linux", feature = "jack-linux"))]
                return crate::jack_backend::run(config, options, process_handler);
                #[cfg(all(target_os = "linux", not(feature = "jack-linux")))]
                {
                    log::error!("The feature \"jack-linux\" is not enabled");
                    return Err(RunConfigError::JackNotEnabledForPlatform);
                }

                #[cfg(all(target_os = "macos", feature = "jack-macos"))]
                return crate::jack_backend::run(config, options, process_handler);
                #[cfg(all(target_os = "macos", not(feature = "jack-macos")))]
                {
                    log::error!("The feature \"jack-macos\" is not enabled");
                    return Err(RunConfigError::JackNotEnabledForPlatform);
                }

                #[cfg(all(target_os = "windows", feature = "jack-windows"))]
                return crate::jack_backend::run(config, options, process_handler);
                #[cfg(all(target_os = "windows", not(feature = "jack-windows")))]
                {
                    log::error!("The feature \"jack-windows\" is not enabled");
                    return Err(RunConfigError::JackNotEnabledForPlatform);
                }
            }
            b => {
                log::error!("Unkown audio backend: {:?}", b);
                return Err(RunConfigError::AudioBackendNotFound(b));
            }
        }
    }
}

/// The handle to a running audio/midi stream.
///
// When this gets dropped, the stream (audio thread) will automatically stop. This
/// is the intended method for stopping a stream.
pub struct StreamHandle<P: ProcessHandler> {
    /// The message channel that recieves notifications from the audio thread
    /// including any errors that have occurred.
    pub messages: StreamMsgChannel,

    pub(crate) platform_handle: Box<dyn PlatformStreamHandle<P>>,
}

impl<P: ProcessHandler> StreamHandle<P> {
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
        in_port_indexes: Vec<usize>,
        out_port_indexes: Vec<usize>,
    ) -> Result<(), ChangeAudioPortsError> {
        self.platform_handle.change_audio_port_config(in_port_indexes, out_port_indexes)
    }

    #[cfg(any(feature = "jack-linux", feature = "jack-macos", feature = "jack-windows"))]
    /// Change the audio port configuration (when using the Jack backend) while the
    /// audio thread is still running.
    ///
    /// This will return an error if the current backend is not Jack.
    pub fn change_jack_audio_port_config(
        &mut self,
        in_port_names: Vec<String>,
        out_port_names: Vec<String>,
    ) -> Result<(), ChangeAudioPortsError> {
        self.platform_handle.change_jack_audio_port_config(in_port_names, out_port_names)
    }

    /// Change the buffer/block size configuration while the audio thread is still
    /// running. Support for this will depend on the backend.
    ///
    /// If the given config is invalid, an error will be returned with no
    /// effect on the running audio thread.
    pub fn change_block_size_config(
        &mut self,
        buffer_size: u32,
    ) -> Result<(), ChangeBlockSizeError> {
        self.platform_handle.change_block_size_config(buffer_size)
    }

    #[cfg(feature = "midi")]
    /// Change the midi device configuration while the audio thread is still running.
    /// Support for this will depend on the backend.
    ///
    /// If the given config is invalid, an error will be returned with no
    /// effect on the running audio thread.
    pub fn change_midi_device_config(
        &mut self,
        in_devices: Vec<MidiPortConfig>,
        out_devices: Vec<MidiPortConfig>,
    ) -> Result<(), ChangeMidiPortsError> {
        self.platform_handle.change_midi_device_config(in_devices, out_devices)
    }

    // It may be possible to also add `change_sample_rate_config()` here, but
    // I'm not sure how useful this would actually be.

    /// Returns whether or not this backend supports changing the audio bus
    /// configuration while the audio thread is running.
    pub fn can_change_audio_ports(&self) -> bool {
        self.platform_handle.can_change_audio_ports()
    }

    // Returns whether or not this backend supports changing the buffer size
    // configuration while the audio thread is running.
    pub fn can_change_block_size(&self) -> bool {
        self.platform_handle.can_change_block_size()
    }

    #[cfg(feature = "midi")]
    /// Returns whether or not this backend supports changing the midi device
    /// config while the audio thread is running.
    pub fn can_change_midi_ports(&self) -> bool {
        self.platform_handle.can_change_midi_ports()
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
    ///
    /// This will return an error if the current backend is Jack. Please use
    /// `change_jack_audio_port_config()` instead for Jack.
    #[allow(unused_variables)]
    fn change_audio_port_config(
        &mut self,
        in_port_indexes: Vec<usize>,
        out_port_indexes: Vec<usize>,
    ) -> Result<(), ChangeAudioPortsError> {
        Err(ChangeAudioPortsError::NotSupportedByBackend)
    }

    #[cfg(any(feature = "jack-linux", feature = "jack-macos", feature = "jack-windows"))]
    /// Change the audio port configuration (when using the Jack backend) while the
    /// audio thread is still running.
    ///
    /// This will return an error if the current backend is not Jack.
    #[allow(unused_variables)]
    fn change_jack_audio_port_config(
        &mut self,
        in_port_names: Vec<String>,
        out_port_names: Vec<String>,
    ) -> Result<(), ChangeAudioPortsError> {
        Err(ChangeAudioPortsError::BackendIsNotJack)
    }

    /// Change the buffer/block size configuration while the audio thread is still
    /// running. Support for this will depend on the backend.
    ///
    /// If the given config is invalid, an error will be returned with no
    /// effect on the running audio thread.
    #[allow(unused_variables)]
    fn change_block_size_config(&mut self, buffer_size: u32) -> Result<(), ChangeBlockSizeError> {
        Err(ChangeBlockSizeError::NotSupportedByBackend)
    }

    #[cfg(feature = "midi")]
    /// Change the midi device configuration while the audio thread is still running.
    /// Support for this will depend on the backend.
    ///
    /// If the given config is invalid, an error will be returned with no
    /// effect on the running audio thread.
    #[allow(unused_variables)]
    fn change_midi_device_config(
        &mut self,
        in_devices: Vec<MidiPortConfig>,
        out_devices: Vec<MidiPortConfig>,
    ) -> Result<(), ChangeMidiPortsError> {
        Err(ChangeMidiPortsError::NotSupportedByBackend)
    }

    // It may be possible to also add `change_sample_rate_config()` here, but
    // I'm not sure how useful this would actually be.

    /// Returns whether or not this backend supports changing the audio bus
    /// configuration while the audio thread is running.
    fn can_change_audio_ports(&self) -> bool {
        false
    }

    // Returns whether or not this backend supports changing the buffer size
    // configuration while the audio thread is running.
    fn can_change_block_size(&self) -> bool {
        false
    }

    #[cfg(feature = "midi")]
    /// Returns whether or not this backend supports changing the midi device
    /// config while the audio thread is running.
    fn can_change_midi_ports(&self) -> bool {
        false
    }
}
