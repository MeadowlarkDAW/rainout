# rainout Design Document

# Objective

The goal of this crate is to provide a powerful, cross-platform, highly configurable, low-latency, and robust solution for connecting to audio and MIDI devices.

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
        - [x] Jack
        - [ ] Pipewire
        - [ ] Alsa (Maybe, depending on how difficult this is. This could be unecessary if Pipewire turns out to be good enough.)
        - [ ] Pulseaudio (Maybe, depending on how difficult this is. This could be unecessary if Pipewire turns out to be good enough.)
    - Mac
        - [x] Jack
        - [ ] CoreAudio
    - Windows
        - [x] Jack
        - [ ] WASAPI
        - [ ] ASIO (reluctantly)
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

The API is divided into four parts: Enumerating the available devices, creating a config, running the stream, and responding to messages after the stream is ran.

## Device Enumeration API:

```rust
/// The list of backends supported by rainout
pub enum Backend {
    Jack,
    Pipewire,
    Alsa,
    CoreAudio,
    Wasapi,
    Asio,
}

/// Returns the list available audio backends for this platform.
///
/// These are ordered with the first item (index 0) being the most highly
/// preferred default backend.
pub fn available_audio_backends() -> &'static [Backend] { ... }

#[cfg(feature = "midi")]
/// Returns the list available midi backends for this platform.
///
/// These are ordered with the first item (index 0) being the most highly
/// preferred default backend.
pub fn available_midi_backends() -> &'static [Backend] { ... }

/// Returns the list of available audio devices for the given backend.
///
/// This will return an error if the backend with the given name could
/// not be found.
pub fn enumerate_audio_backend(backend: Backend) -> Result<AudioBackendOptions, ()> { ... }

/// The name/ID of a device
pub struct DeviceID {
    /// The name of the device
    pub name: String,

    /// The unique identifier of this device (if one is available). This
    /// is usually more reliable than just the name of the device.
    pub identifier: Option<String>,
}

/// Returns the configuration options for the given device.
///
/// This will return an error if the backend or the device could not
/// be found.
pub fn enumerate_audio_device(
    backend: Backend,
    device: &DeviceID,
) -> Result<AudioDeviceConfigOptions, ()> { ... }

#[cfg(any(feature = "jack-linux", feature = "jack-macos", feature = "jack-windows"))]
/// Returns the configuration options for "monolithic" system-wide Jack
/// audio device.
///
/// This will return an error if Jack is not installed on the system
/// or if the Jack server is not running.
pub fn enumerate_jack_audio_device() -> Result<JackAudioDeviceOptions, ()> { ... }

#[cfg(feature = "asio")]
#[cfg(target_os = "windows")]
/// Returns the configuration options for the given ASIO device.
///
/// This will return an error if the device could not be found.
pub fn enumerate_asio_audio_device(device: &DeviceID) -> Result<AsioAudioDeviceOptions, ()> { ... }

#[cfg(feature = "midi")]
/// Returns the list of available midi devices for the given backend.
///
/// This will return an error if the backend with the given name could
/// not be found.
pub fn enumerate_midi_backend(backend: Backend) -> Result<MidiBackendOptions, ()> { ... }

/// Information about an audio backend, including its available devices
/// and configurations
pub struct AudioBackendOptions {
    /// The audio backend
    pub backend: Backend,

    /// The version of this audio backend (if that information is available)
    pub version: Option<String>,

    /// The running status of this backend
    pub status: BackendStatus,

    /// The available audio devices to select from.
    ///
    /// This will be `None` if this backend's `status` is not of the type
    /// `BackendStatus::Running`.
    pub device_options: Option<AudioDeviceOptions>,
}

/// The status of a backend
pub enum BackendStatus {
    /// The backend is installed and running with available devices
    Running,

    /// The backend is installed and running, but no devices were found
    NoDevices,

    /// The backend is not installed on the system and thus cannot be used
    NotInstalled,

    /// The backend is installed but it is not currently running on the system,
    /// and thus cannot be used until it is started
    NotRunning,
}

/// The available audio devices to select from
pub enum AudioDeviceOptions {
    /// Only a single audio device can be selected from this list. These
    /// devices may be output only, input only, or (most commonly)
    /// duplex.
    SingleDeviceOnly {
        /// The available audio devices to select from.
        options: Vec<DeviceID>,
    },

    /// A single input and output device pair can be selected from this list.
    ///
    /// This will only be used by backends that support tying together multiple
    /// audio devices into a single duplex stream (CoreAudio and Pipewire).
    LinkedInOutDevice {
        /// The names/IDs of the available input devices to select from
        in_devices: Vec<DeviceID>,
        /// The names/IDs of the available output devices to select from
        out_devices: Vec<DeviceID>,
    },

    #[cfg(any(feature = "jack-linux", feature = "jack-macos", feature = "jack-windows"))]
    /// There is a single "monolithic" system-wide Jack audio device
    JackSystemWideDevice,

    #[cfg(feature = "asio")]
    /// A single ASIO device can be selected from this list.
    SingleAsioDevice {
        /// A single ASIO device can be selected from this list.
        options: Vec<DeviceID>,
    },
}

/// The available configuration options for the audio device/devices
pub struct AudioDeviceConfigOptions {
    /// The available sample rates to choose from.
    ///
    /// If the available sample rates could not be determined at this time,
    /// then this will be `None`.
    pub sample_rates: Option<Vec<u32>>,

    /// The available range of fixed block/buffer sizes
    ///
    /// If the device does not support fixed block/buffer sizes, then this
    /// will be `None`.
    pub block_sizes: Option<BlockSizeRange>,

    /// The number of input audio channels
    pub num_in_channels: usize,
    /// The number of output audio channels
    pub num_out_channels: usize,

    /// The layout of the input audio channels
    pub in_channel_layout: ChannelLayout,
    /// The layout of the output audio channels
    pub out_channel_layout: ChannelLayout,

    /// If `true` then it means that the application can request to take
    /// exclusive access of the device to improve latency.
    ///
    /// This is only relevant for WASAPI on Windows. This will always be
    /// `false` on other backends and platforms.
    pub can_take_exclusive_access: bool,
}

/// The channel layout of the audio ports
pub enum ChannelLayout {
    /// The device has not specified the channel layout of the audio ports
    Unspecified,
    /// The device has a single mono channel
    Mono,
    /// The device has multiple mono channels (i.e. multiple microphone
    /// inputs)
    MultiMono,
    /// The device has a single stereo channel
    Stereo,
    /// The device has multiple stereo channels (i.e. multiple stereo outputs
    /// such as an output for speakers and another for headphones)
    MultiStereo,
    /// Some other configuration not listed.
    Other(String),
    // TODO: More channel layouts
}

/// The range of possible block sizes for an audio device.
pub struct BlockSizeRange {
    /// The minimum buffer/block size that can be used (inclusive)
    pub min: u32,

    /// The maximum buffer/block size that can be used (inclusive)
    pub max: u32,

    /// The default buffer/block size for this device
    pub default: u32,
}

#[cfg(any(feature = "jack-linux", feature = "jack-macos", feature = "jack-windows"))]
/// Information and configuration options for the "monolithic" system-wide
/// Jack audio device
pub struct JackAudioDeviceOptions {
    /// The sample rate of the Jack device
    pub sample_rate: u32,

    /// The block size of the Jack device
    pub block_size: u32,

    /// The names of the available input ports to select from
    pub in_ports: Vec<String>,
    /// The names of the available output ports to select from
    pub out_ports: Vec<String>,

    /// The indexes of the default input ports (into the Vec `in_ports`)
    ///
    /// If no default input ports could be found, then this will be `None`.
    pub default_in_ports: Option<Vec<usize>>,
    /// The indexes of the default output ports (into the Vec `out_ports`)
    ///
    /// If no default output ports could be found, then this will be `None`.
    pub default_out_ports: Option<Vec<usize>>,
}

#[cfg(feature = "asio")]
/// Information and configuration options for an ASIO audio device on
/// Windows
pub struct AsioAudioDeviceOptions {
    /// The configuration options for this ASIO audio device
    pub config_options: AudioDeviceConfigOptions,

    /// The path the the executable that launches the settings GUI for
    /// this ASIO device
    pub settings_app: std::path::PathBuf,
}

#[cfg(feature = "midi")]
/// Information about a MIDI backend, including its available devices
/// and configurations
pub struct MidiBackendOptions {
    /// The MIDI backend
    pub backend: Backend,

    /// The version of this MIDI backend (if that information is available)
    pub version: Option<String>,

    /// The running status of this backend
    pub status: BackendStatus,

    /// The names of the available input MIDI ports to select from
    pub in_ports: Vec<MidiPortOptions>,
    /// The names of the available output MIDI ports to select from
    pub out_ports: Vec<MidiPortOptions>,

    /// The index of the default/preferred input MIDI port for the backend
    ///
    /// This will be `None` if no default input port could be
    /// determined.
    pub default_in_port: Option<usize>,
    /// The index of the default/preferred output MIDI port for the backend
    ///
    /// This will be `None` if no default output port could be
    /// determined.
    pub default_out_port: Option<usize>,
}

#[cfg(feature = "midi")]
/// Information and configuration options for a MIDI device port
pub struct MidiPortOptions {
    /// The name/ID of this device
    pub id: DeviceID,

    /// The index of this port for this device
    pub port_index: usize,

    /// The type of control scheme that this port uses
    pub control_type: MidiControlScheme,
}

#[cfg(feature = "midi")]
/// The type of control scheme that this port supports
pub enum MidiControlScheme {
    /// Supports only MIDI version 1
    Midi1,

    #[cfg(feature = "midi2")]
    /// Supports MIDI version 2 (and by proxy also supports MIDI version 1)
    Midi2,
    // TODO: Midi versions inbetween 1.0 and 2.0?
    // TODO: OSC devices?
}
```

## Configuration API:

This is the API for the "configuration". The user constructs this configuration in whatever method they choose (from a settings GUI or a config file) and sends it to this crate to be ran.

```rust
/// Specifies whether to use a specific configuration or to automatically
/// select the best configuration.
pub enum AutoOption<T: Debug + Clone + PartialEq> {
    /// Use this specific configuration.
    Use(T),

    /// Automatically select the best configuration.
    Auto,
}

/// The configuration of audio and MIDI backends and devices.
pub struct RainoutConfig {
    /// The audio backend to use.
    ///
    /// Set this to `AutoOption::Auto` to automatically select the best
    /// backend to use.
    pub audio_backend: AutoOption<Backend>,

    /// The audio device/devices to use.
    ///
    /// Set this to `AudioDeviceConfig::Auto` to automatically select the best
    /// audio device to use.
    pub audio_device: AudioDeviceConfig,

    /// The sample rate to use.
    ///
    /// Set this to `AutoOption::Auto` to automatically select the best
    /// sample rate to use.
    pub sample_rate: AutoOption<u32>,

    /// The block/buffer size to use.
    ///
    /// Set this to `AutoOption::Auto` to automatically select the best
    /// buffer/block size to use.
    pub block_size: AutoOption<u32>,

    /// If `true` then it means that the application can request to take
    /// exclusive access of the device to improve latency.
    ///
    /// This is only relevant for WASAPI on Windows. This will always be
    /// `false` on other backends and platforms.
    ///
    /// By default this is set to `false`.
    pub take_exclusive_access: bool,

    #[cfg(feature = "midi")]
    /// The configuration of MIDI devices.
    ///
    /// Set this to `None` to use no MIDI devices.
    pub midi_config: Option<MidiConfig>,
}

/// The configuration of which audio device/devices to use.
pub enum AudioDeviceConfig {
    /// Use a single audio device. These device may be output only, input
    /// only, or (most commonly) duplex.
    Single(DeviceID),

    /// Use an input/output device pair. This is only supported on some
    /// backends.
    LinkedInOut { input: Option<DeviceID>, output: Option<DeviceID> },

    #[cfg(any(feature = "jack-linux", feature = "jack-macos", feature = "jack-windows"))]
    /// When the audio backend is Jack, the names of the audio ports to use.
    ///
    /// This is only relevent when the audio backend is Jack.
    Jack {
        /// The names of the audio input ports to use.
        ///
        /// The buffers presented in `ProcInfo::audio_in` will appear in the
        /// exact same order as this Vec.
        ///
        /// If a port with the given name does not exist, then an unconnected
        /// virtual port with that same name will be created.
        ///
        /// You may also pass in an empty Vec to have no audio inputs.
        in_ports: Vec<String>,

        /// The names of the audio output ports to use.
        ///
        /// The buffers presented in `ProcInfo::audio_out` will appear in the
        /// exact same order as this Vec.
        ///
        /// If a port with the given name does not exist, then an unconnected
        /// virtual port with that same name will be created.
        ///
        /// You may also pass in an empty Vec to have no audio outputs.
        out_ports: Vec<String>,
    },

    /// Automatically select the best configuration.
    Auto,
}

#[cfg(feature = "midi")]
/// The configuration of the MIDI backend and devices.
pub struct MidiConfig {
    /// The MIDI backend to use.
    ///
    /// Set this to `AutoOption::Auto` to automatically select the best
    /// backend to use.
    pub midi_backend: AutoOption<Backend>,

    /// The names of the MIDI input ports to use.
    ///
    /// The buffers presented in `ProcInfo::midi_in` will appear in the
    /// exact same order as this Vec.
    ///
    /// Set this to `AutoOption::Auto` to automatically select the best
    /// configuration of input ports to use.
    ///
    /// You may also pass in an empty Vec to have no MIDI inputs.
    pub in_ports: AutoOption<Vec<MidiPortConfig>>,

    /// The names of the MIDI output ports to use.
    ///
    /// The buffers presented in `ProcInfo::midi_out` will appear in the
    /// exact same order as this Vec.
    ///
    /// Set this to `AutoOption::Auto` to automatically select the best
    /// configuration of output ports to use.
    ///
    /// You may also pass in an empty Vec to have no MIDI outputs.
    pub out_ports: AutoOption<Vec<MidiPortConfig>>,
}

#[cfg(feature = "midi")]
/// The configuration of a MIDI device port
pub struct MidiPortConfig {
    /// The name/ID of the MIDI device to use
    pub device_id: DeviceID,

    /// The index of the port on the device
    pub port_index: usize,

    /// The control scheme to use for this port
    pub control_scheme: MidiControlScheme,
}

```

## Running API:

The user sends a config to this API to run it.

```rust
/// Get the estimated sample rate and total latency of a particular configuration
/// before running it.
///
/// `None` will be returned if the sample rate or latency is not known at this
/// time.
///
/// `(Option<SAMPLE_RATE>, Option<LATENCY>)`
pub fn estimated_sample_rate_and_latency(
    config: &RainoutConfig,
) -> Result<(Option<u32>, Option<u32>), RunConfigError> { ... }

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

/// Additional options for running a stream
pub struct RunOptions {
    /// If `Some`, then the backend will use this name as the
    /// client name that appears in the audio server. This is only relevent for some
    /// backends like Jack.
    ///
    /// By default this is set to `None`.
    pub use_application_name: Option<String>,

    /// If this is `true`, then the system will try to automatically connect to
    /// the default audio input channels when using `AutoOption::Auto`.
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
    /// connect to a device with at-least two output channels. It will also try
    /// to avoid automatically connecting to devices with mono outputs.
    ///
    /// By default this is set to `true`.
    pub must_have_stereo_output: bool,

    /// If `true`, then the system will use empty (silent) buffers for any
    /// audio/MIDI channels/ports that failed to connect instead of returning an
    /// error.
    ///
    /// By default this is set to `false`.
    pub empty_buffers_for_failed_ports: bool,

    /// If the audio backend does not support fixed buffer sizes, use this as
    /// the maximum size of the audio buffers passed to the process method.
    ///
    /// By default this is set to `1024`.
    pub max_buffer_size: usize,

    /// The size of the audio thread to stream handle message buffer.
    ///
    /// By default this is set to `512`.
    pub msg_buffer_size: usize,
}

/// Run the given configuration in an audio thread.
///
/// * `config`: The configuration to use.
/// * `options`: Various options for the stream.
/// * `process_handler`: An instance of your process handler.
///
/// If an error is returned, then it means the config failed to run and no audio
/// thread was spawned.
pub fn run<P: ProcessHandler>(
    config: &Config,
    options: &RunOptions,
    process_handler: P,
) -> Result<StreamHandle<P>, RunConfigError> { ... }

/// The handle to a running audio/midi stream.
///
// When this gets dropped, the stream (audio thread) will automatically stop. This
/// is the intended method for stopping a stream.
pub struct StreamHandle<P: ProcessHandler, E: ErrorHandler> {
    /// The message channel that recieves notifications from the audio thread
    /// including any errors that have occurred.
    pub messages: ringbuf::Consumer<StreamMsg>,

    ...
}

impl<P: ProcessHandler, E: ErrorHandler> StreamHandle<P, E> {
    /// Returns the actual configuration of the running stream. This may differ
    /// from the configuration passed into the `run()` method.
    pub fn stream_info(&self) -> &StreamInfo { ... }

    #[cfg(any(feature = "jack-linux", feature = "jack-macos", feature = "jack-windows"))]
    /// Change the audio port configuration (when using the Jack backend) while the
    /// audio thread is still running.
    ///
    /// This will return an error if the current backend is not Jack.
    pub fn change_jack_audio_ports(
        &mut self,
        in_port_names: Vec<String>,
        out_port_names: Vec<String>,
    ) -> Result<(), ()> { ... }

    /// Change the buffer/block size configuration while the audio thread is still
    /// running. Support for this will depend on the backend.
    ///
    /// If the given config is invalid, an error will be returned with no
    /// effect on the running audio thread.
    pub fn change_block_size(
        &mut self,
        buffer_size: u32,
    ) -> Result<(), ChangeBlockSizeError> { ... }

    #[cfg(feature = "midi")]
    /// Change the midi device configuration while the audio thread is still running.
    /// Support for this will depend on the backend.
    ///
    /// If the given config is invalid, an error will be returned with no
    /// effect on the running audio thread.
    pub fn change_midi_ports(
        &mut self,
        in_devices: Vec<MidiPortConfig>,
        out_devices: Vec<MidiPortConfig>,
    ) -> Result<(), ChangeMidiPortsError> { ... }

    // Returns whether or not this backend supports changing the buffer size
    // configuration while the audio thread is running.
    pub fn can_change_block_size(&self) -> bool { ... }

    #[cfg(feature = "midi")]
    /// Returns whether or not this backend supports changing the midi device
    /// config while the audio thread is running.
    pub fn can_change_midi_ports(&self) -> bool { ... }
}

// TODO: Implementations of `RunConfigErrorRunConfigError`, `ChangeAudioPortConfigError`,
// and `ChangeBufferSizeConfigError`, and `ChangeMidiDeviceConfigError`.
```

## Message channel API:

After a stream is ran, the user then listens to and responds to events sent to `StreamHandle::messages`.

```rust
pub enum StreamMsg {
    /// An audio device was unplugged while the stream was running. Any connected
    /// ports will input/output silence.
    AudioDeviceDisconnected(DeviceID),

    /// An audio device was reconnected while the stream was running. Any connected
    /// ports will function properly now.
    /// 
    /// This will only be sent after an `AudioDeviceDisconnected` event.
    AudioDeviceReconnected(DeviceID),

    #[cfg(feature = "midi")]
    /// The MIDI output device was not found. This port will produce no MIDI events.
    MidiDeviceDisconnected(DeviceID),

    #[cfg(feature = "midi")]
    /// A MIDI device was reconnected while the stream was running. Any connected
    /// ports will function properly now.
    /// 
    /// This will only be sent after an `MidiDeviceDisconnected` event.
    MidiDeviceReconnected(DeviceID),

    /// An error that caused the stream to close. Please discard this Stream Handle
    /// channel and prepare to start a new stream.
    Error(StreamError),

    /// The audio stream was closed gracefully. Please discard this Stream Handle.
    Closed,
}
// See code in the repo for the implementation of `StreamError`.
```

# Demo Application

In addition to the main API, we will also have a full-working demo application with a working settings GUI. This will probably be written in `egui`, but another UI toolkit could be used.

# Example Settings GUI Logic
```
Audio Settings TAB:

<DROPDOWN> - Select audio backend (JACK, ASIO, WINAPI, COREAUDIO, etc) {
    (if the backend is not installed (only relevant for JACK and Pipewire)) {
        <TEXT> - "Backend not installed"
    } (else if the backend is not running (only relevant for JACK)) {
        <BUTTON> - "Activate backend"
    } (else) {
        (if backend has version info) {
            <TEXT> - "Version: {}"
        }

        (if backend is JACK) {
            <TEXT> - "Sample rate: {}"
            <TEXT> - "Block size: {}"

            <PANEL> - Input ports {
                <BUTTON> - "Add port"
                <LIST> {
                    <DROPDOWN> - port name
                    <BUTTON> - remove port
                }
            }
            <PANEL> - Output ports {
                <BUTTON> - "Add port"
                <LIST> {
                    <DROPDOWN> - port name
                    <BUTTON> - remove port
                }
            }
            <BUTTON> - "Reset to default ports"
        } (else) {
            (if backend is ASIO) {
                <DROPDOWN> - Select a single audio device {
                    <BUTTON> - "Open ASIO settings window"

                    <TEXT> - "Sample rate: {}"
                    <TEXT> - "Block size: {}"
                } (else if no device available) {
                    <TEXT> - "No audio devices detected"
                } (else if no device selected) {
                    <TEXT> - "Please select and audio device from the list"
                }
            } (else if backend supports linking a separate input and output device together) {
                <DROPDOWN> - Select input audio device
                <DROPDOWN> - Select output audio device

                (if one or both devices selected) {
                    <NUMBER SELECTOR> - "Block size" (this will have a minumum and maximum value)
                    <BUTTON> - Reset to default block size

                    <DROPDOWN> - Sample rate
                    <BUTTON> - Reset to default sample rate
                } (else if no device available) {
                    <TEXT> - "No audio devices detected"
                } (else if no device selected) {
                    <TEXT> - "Please select and audio device from the list"
                }
            } (else) {
                <DROPDOWN> - Select a single audio device {
                    <NUMBER SELECTOR> - "Block size" (this will have a minumum and maximum value)
                    <BUTTON> - Reset to efault block size

                    <DROPDOWN> - Sample rate
                    <BUTTON> - Reset to default sample rate

                    (if backend is WASAPI) {
                        <Checkbox> - "Take exclusive access"
                    }
                } (else if no device available) {
                    <TEXT> - "No audio devices detected"
                } (else if no device selected) {
                    <TEXT> - "Please select and audio device from the list"
                }
            }

            <PANEL> - Input busses {
                <BUTTON> - "Add bus"
                <LIST> {
                    <LIST> {
                        <A checkbox for each input port>
                    }
                    <BUTTON> - remove bus
                }
            }

            <PANEL> - Output busses {
                <BUTTON> - "Add bus"
                <LIST> {
                    <LIST> {
                        <A checkbox for each input port>
                    }
                    <BUTTON> - remove bus
                }
            }
        }
    }
}

MIDI Settings TAB:

<DROPDOWN> - Select MIDI backend (JACK, ASIO, WINAPI, COREAUDIO, etc) {
    (if the backend is not installed (only relevant for JACK and Pipewire)) {
        <TEXT> - "Backend not installed"
    } (else if the backend is not running (only relevant for JACK)) {
        <BUTTON> - "Activate backend"
    } (else) {
        (if backend has version info) {
            <TEXT> - "Version: {}"
        }

        <PANEL> - Input ports {
            <BUTTON> - "Add input"
            <LIST> {
                <DROPDOWN> - port name
                <CHECKBOX> - "Follow selection"
                (if port supports MIDI2) {
                    <CHECKBOX> - "Use MIDI2"
                }
                <BUTTON> - remove port
            }

            <BUTTON> - "Reset to default"
        }
        <PANEL> - Output ports {
            <BUTTON> - "Add output"
            <LIST> {
                <DROPDOWN> - port name
                (if port supports MIDI2) {
                    <CHECKBOX> - "Use MIDI2"
                }
                <BUTTON> - remove port
            }

            <BUTTON> - "Reset to default"
        }
    }
}
```
