# rusty-daw-io Design Document

*(note "rusty-daw-io" may not be the final name of this crate)*

# Objective

The goal of this crate is to provide a powerful, cross-platform, highly configurable, low-latency, and robust solution for connecting audio software to audio and MIDI devices.

## Why not contribute to an already existing project like `RTAudio` or `CPAL`?

### RTAudio
- This API is written in a complicated C++ codebase, making it very tricky to bind to other languages such as Rust.
- This project has a poor track record in its stability and ability to gracefully handle errors (not ideal for live audio software).

### CPAL
In short, CPAL is very opinionated, and we have a few deal-breaking issues with its core design.

- CPAL's design does not handle duplex audio devices well. It spawns each input and output stream into separate threads, requiring the developer to sync them together with ring buffers. This is inneficient for most consumer and professional duplex audio devices which already have their inputs and outputs tied into the same stream to reduce latency.
- The API for searching for and configuring audio devices is cumbersome. It returns a list of every possible combination of configurations available with the system's devices. This is not how a user configuring audio settings through a GUI expects this to work.
- CPAL does not have any support for MIDI devices, so we would need to write our own support for it anyway.

Why not just fork `CPAL`?
- To fix these design issues we would pretty much need to rewrite the whole API anyway. Of course we don't have to work completely from scratch. We can still borrow some of the low-level platform specific code in CPAL.

# Goals
- Support for Linux, Mac, and Windows using the following backends: (and maybe Android and iOS in the future, but that is not a gaurantee)
    - Linux
        - [ ] Jack
        - [ ] Pipewire
        - [ ] Alsa (Maybe, depending on how difficult this is. This could be unecessary if Pipewire turns out to be good enough.)
        - [ ] Pulseaudio (Maybe, depending on how difficult this is. This could be unecessary if Pipewire turns out to be good enough.)
    - Mac
        - [ ] CoreAudio
        - [ ] Jack (Maybe, if it is stable enough on Mac.)
    - Window
        - [ ] WASAPI
        - [ ] ASIO (reluctantly)
        - [ ] Jack (Maybe, if it is stable enough on Windows.)
- Scan the available devices on the system, and present configuration options in a format that is intuitive to an end-user configuring devices inside a settings GUI.
- Send all audio and midi streams into a single high-priority thread, taking advantage of native duplex devices when available. (Audio buffers will be presented as de-interlaced `f32` buffers).
- Robust and graceful error handling, especially while the stream is running.
- Easily save and load configurations to/from a config file.
- A system that will try to automatically create a good initial default configuration.

# Later/Maybe Goals
- Support MIDI 2.0 devices
- Support for OSC devices
- C API bindings

# Non-Goals
- No Android and iOS support (for now atleast)
- No support for using multiple backends at the same time (i.e trying to use WASAPI device as an input and an ASIO device as an output). This will just add a whole slew of complexity and stuff that can go wrong.
- No support for tying multiple separate (non-duplexed) audio devices together. We will only support either connecting to a single duplex audio device *or* connecting to a single non-duplex output device.
    - This one is probably controversal, so let me explain the reasoning:
        - Pretty much all modern external audio devices (a setup used by most professionals and pro-sumers) are already duplex.
        - MacOS (and in Linux using JACK or Pipewire) already packages all audio device streams into a single "system-wide duplex device". So this is really only a Windows-specific problem.
        - Tying together multiple non-duplex audio streams requires an intermediate buffer that adds a sometimes unkowable amount of latency.
        - Allowing for multiple separate audio devices adds a lot of complexity to both the settings GUI and the config file, and a lot more that can go wrong.
        - Some modern DAWs like Bitwig already use this "single audio device only" system, so it's not like it's a new concept.
- No support for non-f32 audio streams.
    - There is just no point in my opinion in presenting any other sample format other than `f32` in such an API. These `f32` buffers will just be converted to/from the native sample format that the device wants behind the scenes.

# API Design

The API is divided into three stages: Enumerating the available devices, creating a config, and running the stream.

## Device Enumeration API:

```rust
/// Returns the available audio backends for this platform.
///
/// These are ordered with the first item (index 0) being the most highly
/// preferred default backend.
pub fn available_audio_backends() -> &'static [AudioBackend] {
    platform::available_audio_backends()
}

#[cfg(feature = "midi")]
/// Returns the available midi backends for this platform.
///
/// These are ordered with the first item (index 0) being the most highly
/// preferred default backend.
pub fn available_midi_backends() -> &'static [MidiBackend] {
    platform::available_midi_backends()
}

/// Get information about a particular audio backend and its devices.
///
/// This will update the list of available devices as well as the the
/// status of whether or not this backend is running.
///
/// This will return an error if the backend is not available on this system.
pub fn enumerate_audio_backend(backend: AudioBackend) -> Result<AudioBackendInfo, ()> {
    platform::enumerate_audio_backend(backend)
}

#[cfg(feature = "midi")]
/// Get information about a particular midi backend and its devices.
///
/// This will update the list of available devices as well as the the
/// status of whether or not this backend is running.
///
/// This will return an error if the backend is not available on this system.
pub fn enumerate_midi_backend(backend: MidiBackend) -> Result<MidiBackendInfo, ()> {
    platform::enumerate_midi_backend(backend)
}

/// Enumerate through each backend to find the preferred/best default audio
/// backend for this system.
///
/// If a higher priority backend does not have any available devices, then
/// this will try to return the next best backend that does have an
/// available device.
///
/// This does not enumerate through the devices in each backend, just the
/// names of each device.
pub fn find_preferred_audio_backend() -> AudioBackend {
    ...
}

/// Information about a particular audio backend, including a list of the
/// available devices.
#[derive(Debug, Clone)]
pub struct AudioBackendInfo {
    /// The type of backend.
    pub backend: AudioBackend,

    /// The version of this backend (if there is one available)
    ///
    /// (i.e. "1.2.10")
    pub version: Option<String>,

    /// If this is true, then it means this backend is running on this system.
    /// (For example, if this backend is Jack and the Jack server is not currently
    /// running on the system, then this will be false.)
    pub running: bool,

    /// The devices that are available in this backend.
    ///
    /// Please note that these are not necessarily each physical device in the
    /// system. For example, in backends like Jack and CoreAudio, the whole system
    /// acts like a single "duplex device" which is the audio server itself.
    pub devices: Vec<AudioDeviceInfo>,

    /// The index of the preferred/best default device for this backend.
    ///
    /// This will be `None` if the preferred device is not known at this time.
    pub default_device: Option<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeviceID {
    /// The name of this device.
    pub name: String,

    /// The unique identifier of this device (if one is available).
    ///
    /// This is usually more reliable than just using the name of
    /// the device.
    pub unique_id: Option<String>,
}

/// An audio backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioBackend {
    /// Pipewire on Linux
    Pipewire,
    #[cfg(feature = "jack-linux")]
    /// Jack on Linux
    JackLinux,
    #[cfg(feature = "alsa")]
    /// Alsa on Linux
    Alsa,
    #[cfg(feature = "pulseaudio")]
    /// Pulseaudio on Linux
    Pulseaudio,
    /// CoreAudio on Mac
    CoreAudio,
    #[cfg(feature = "jack-macos")]
    /// Jack on MacOS
    JackMacOS,
    /// WASAPI on Windows
    Wasapi,
    #[cfg(feature = "asio")]
    /// ASIO on Windows
    Asio,
    #[cfg(feature = "jack-windows")]
    /// Jack on Windows
    JackWindows,
}

impl AudioBackend {
    /// If this is true, then it means it is relevant to actually show the available
    /// devices as a list to select from in a settings GUI.
    ///
    /// In backends like Jack and CoreAudio which set this to false, there is only
    /// ever one "system-wide duplex device" which is the audio server itself, and
    /// thus showing this information in a settings GUI is irrelevant.
    pub fn devices_are_relevant(&self) -> bool {
        ...
    }

    /// If this is true, then it means that this backend supports creating
    /// virtual ports that can be connected later.
    pub fn supports_creating_virtual_ports(&self) -> bool {
        ...
    }
}

#[cfg(feature = "midi")]
/// A midi backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MidiBackend {
    /// Pipewire on Linux
    Pipewire,
    #[cfg(feature = "jack-linux")]
    /// Jack on Linux
    JackLinux,
    #[cfg(feature = "alsa")]
    /// Alsa on Linux
    Alsa,
    #[cfg(feature = "pulseaudio")]
    /// Pulseaudio on Linux
    Pulseaudio,
    /// CoreAudio on Mac
    CoreAudio,
    #[cfg(feature = "jack-macos")]
    /// Jack on MacOS
    JackMacOS,
    /// WASAPI on Windows
    Wasapi,
    #[cfg(feature = "asio")]
    /// ASIO on Windows
    Asio,
    #[cfg(feature = "jack-windows")]
    /// Jack on Windows
    JackWindows,
}

/// Information about a particular audio device, including all its available
/// configurations.
#[derive(Debug, Clone)]
pub struct AudioDeviceInfo {
    pub id: DeviceID,

    /// The names of the available input ports (one port per channel) on this device
    /// (i.e. "mic_1", "mic_2", "system_input", etc.)
    pub in_ports: Vec<String>,

    /// The names of the available output ports (one port per channel) on this device
    /// (i.e. "out_1", "speakers_out_left", "speakers_out_right", etc.)
    pub out_ports: Vec<String>,

    /// The available sample rates for this device.
    ///
    /// This is irrelevant for ASIO devices because the buffer size is configured
    /// through the configuration GUI application for that device.
    pub sample_rates: Vec<u32>,

    /// The default/preferred sample rate for this audio device.
    ///
    /// This is irrelevant for ASIO devices because the buffer size is configured
    /// through the configuration GUI application for that device.
    pub default_sample_rate: u32,

    /// The supported range of fixed buffer/block sizes for this device. If the device
    /// doesn't support fixed-size buffers then this will be `None`.
    ///
    /// This is irrelevant for ASIO devices because the buffer size is configured
    /// through the configuration GUI application for that device.
    pub fixed_buffer_size_range: Option<FixedBufferSizeRange>,

    /// The default channel layout of the input ports for this device.
    pub default_input_layout: DefaultChannelLayout,

    /// The default channel layout of the output ports for this device.
    pub default_output_layout: DefaultChannelLayout,

    #[cfg(feature = "asio")]
    /// If this audio device is an ASIO device, then this will contain extra
    /// information about the device.
    pub asio_info: Option<AsioDeviceInfo>,
}

#[cfg(feature = "asio")]
#[derive(Debug, Clone)]
pub struct AsioDeviceInfo {
    /// The path to the configuration GUI application for the device.
    pub config_gui_path: std::path::PathBuf,

    /// The sample rate that has been configured for this device.
    ///
    /// You will need to re-enumerate this device to get the new sample
    /// rate after configuring through the device's configuration GUI
    /// application.
    pub sample_rate: u32,

    /// The fixed buffer size that has been configured for this device.
    ///
    /// You will need to re-enumerate this device to get the new sample
    /// rate after configuring through the device's configuration GUI
    /// application.
    pub fixed_buffer_size: u32,
}

/// The range of possible fixed sizes of buffers/blocks for an audio device.
#[derive(Debug, Clone)]
pub struct FixedBufferSizeRange {
    /// The minimum buffer/block size (inclusive)
    pub min: u32,
    /// The maximum buffer/block size (inclusive)
    pub max: u32,

    /// If this is `true` then it means the device only supports fixed buffer/block
    /// sizes between `min` and `max` that are a power of 2.
    pub must_be_power_of_2: bool,

    /// The default/preferred fixed buffer size for this device.
    pub default: u32,
}

/// The default channel layout of the ports for an audio device.
///
/// These include the index of each port for each channel.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum DefaultChannelLayout {
    /// The device has not specified the default channel layout of its ports.
    Unspecified,

    Mono(usize),
    Stereo {
        left: usize,
        right: usize,
    },

    // TODO: More channel layouts
}

#[cfg(feature = "midi")]
/// Information about a particular midi backend, including a list of the
/// available devices.
#[derive(Debug, Clone)]
pub struct MidiBackendInfo {
    /// The type of backend.
    pub backend: MidiBackend,

    /// The version of this backend (if there is one available)
    ///
    /// (i.e. "1.2.10")
    pub version: Option<String>,

    /// If this is true, then it means this backend is running on this system.
    /// (For example, if this backend is Jack and the Jack server is not currently
    /// running on the system, then this will be false.)
    pub running: bool,

    /// The list of available input MIDI devices
    pub in_devices: Vec<MidiDeviceInfo>,

    /// The list of available output MIDI devices
    pub out_devices: Vec<MidiDeviceInfo>,

    /// The index of the preferred/best default input device for this backend.
    ///
    /// This will be `None` if the preferred device is not known at this time.
    pub default_in_device: Option<usize>,

    /// The index of the preferred/best default output device for this backend.
    ///
    /// This will be `None` if the preferred device is not known at this time.
    pub default_out_device: Option<usize>,
}

#[cfg(feature = "midi")]
/// Information about a particular midi device, including all its available
/// configurations.
#[derive(Debug, Clone)]
pub struct MidiDeviceInfo {
    pub id: DeviceID,
    // TODO: More information about the MIDI device
}
```

## Configuration API:

This is the API for the "configuration". The user constructs this configuration in whatever method they choose (from a settings GUI or a config file) and sends it to this crate to be ran.

```rust
/// A full configuration of audio and midi devices to connect to.
#[derive(Debug, Clone)]
pub struct Config {
    /// The type of the audio backend to use.
    pub audio_backend: AudioBackend,

    /// The ID of the audio device to use.
    pub audio_device: DeviceID,

    /// The names of the audio input ports to use.
    ///
    /// The buffers presented in the `ProcessInfo::audio_inputs` will appear in this exact same
    /// order.
    pub audio_in_ports: Vec<String>,

    /// The names of the audio output ports to use.
    ///
    /// The buffers presented in the `ProcessInfo::audio_outputs` will appear in this exact same
    /// order.
    pub audio_out_ports: Vec<String>,

    /// The sample rate to use.
    pub sample_rate: u32,

    /// The buffer size configuration for this device.
    pub buffer_size: AudioBufferSizeConfig,

    #[cfg(feature = "midi")]
    /// The configuration for MIDI devices.
    ///
    /// Set this to `None` to use no MIDI devices in the stream.
    pub midi_config: Option<MidiConfig>,
}

/// The buffer size configuration for an audio device.
#[derive(Debug, Clone, Copy)]
pub struct AudioBufferSizeConfig {
    /// If `Some`, then the backend will attempt to use a fixed size buffer of the
    /// given size. If this is `None`, then the backend will attempt to use the default
    /// fixed buffer size (if there is one).
    pub try_fixed_buffer_size: Option<u32>,

    /// If the backend fails to set a fixed buffer size from `try_fixed_buffer_size`,
    /// then unfixed buffer sizes will be used instead. This number will be the
    /// maximum size of a buffer that will be passed into the `process()` method in
    /// that case.
    pub fallback_max_buffer_size: u32,
}

#[cfg(feature = "midi")]
/// A full configuration of midi devices to connect to.
#[derive(Debug, Clone)]
pub struct MidiConfig {
    /// The type of the audio backend to use.
    pub backend: MidiBackend,

    /// The IDs of the input MIDI devices to use.
    ///
    /// The buffers presented in the `ProcessInfo::midi_inputs` will appear in this exact same
    /// order.
    pub in_devices: Vec<DeviceID>,

    /// The IDs of the output MIDI devices to use.
    ///
    /// The buffers presented in the `ProcessInfo::midi_outputs` will appear in this exact
    /// same order.
    pub out_devices: Vec<DeviceID>,
}
```

## Running API:

The user sends a config to this API to run it.

```rust
/// Get the estimated total latency of a particular configuration before running it.
///
/// `None` will be returned if the latency is not known at this time or if the
/// given config is invalid.
pub fn estimated_latency(config: &Config) -> Option<u32> {
    ...
}

/// Get the sample rate of a particular configuration before running it.
///
/// `None` will be returned if the sample rate is not known at this time or if the
/// given config is invalid.
pub fn sample_rate(config: &Config) -> Option<u32> {
    ...
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

// See code in the repo for the implementations of `StreamInfo` and `ProcessInfo`.

/// An error handler for a stream.
pub trait ErrorHandler: 'static + Send + Sync {
    /// Called when a non-fatal error occurs (any error that does not require the audio
    /// thread to restart).
    fn nonfatal_error(&mut self, error: StreamError);

    /// Called when a fatal error occurs (any error that requires the audio thread to
    /// restart).
    fn fatal_error(self, error: FatalStreamError);
}

// TODO: Implementations of `StreamError` and `FatalStreamError`.

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
    ...
}

impl<P: ProcessHandler, E: ErrorHandler> StreamHandle<P, E> {
    /// Returns the actual configuration of the running stream. This may differ
    /// from the configuration passed into the `run()` method.
    pub fn stream_info(&self) -> &StreamInfo {
        ...
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
        ...
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
        ...
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
        ...
    }

    // It may be possible to also add `change_sample_rate_config()` here, but
    // I'm not sure how useful this would actually be.

    /// Returns whether or not this backend supports changing the audio bus
    /// configuration while the audio thread is running.
    pub fn can_change_audio_port_config(&self) -> bool {
        ...
    }

    // Returns whether or not this backend supports changing the buffer size
    // configuration while the audio thread is running.
    pub fn can_change_audio_buffer_size_config(&self) -> bool {
        ...
    }

    #[cfg(feature = "midi")]
    /// Returns whether or not this backend supports changing the midi device
    /// config while the audio thread is running.
    pub fn can_change_midi_device_config(&self) -> bool {
        ...
    }
}

// TODO: Implementations of `RunConfigErrorRunConfigError`, `ChangeAudioPortConfigError`,
// and `ChangeBufferSizeConfigError`, and `ChangeMidiDeviceConfigError`.
```

# Demo Application

In addition to the main API, we will also have a full-working demo application with a working settings GUI. This will probably be written in `egui`, but another UI toolkit could be used.