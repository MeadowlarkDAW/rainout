use serde::{Deserialize, Serialize};

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "linux")]
pub use linux::*;

pub mod midi_buffer;
pub use midi_buffer::*;

pub const MAX_MIDI_MESSAGES: usize = 2048;
pub const MAX_MIDI_MSG_SIZE: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BufferSizeInfo {
    MaximumSize(u32),
    ConstantSize(u32),
    UnknownSize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SystemAudioDeviceInfo {
    pub name: String,
    pub ports: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AudioServerInfo {
    pub name: String,
    pub version: Option<String>,
    pub system_in_devices: Vec<SystemAudioDeviceInfo>,
    pub system_out_devices: Vec<SystemAudioDeviceInfo>,
    pub sample_rates: Vec<u32>,
    pub buffer_size: BufferSizeInfo,
    pub active: bool,
}

impl AudioServerInfo {
    pub(crate) fn new(name: String, version: Option<String>) -> Self {
        Self {
            name,
            version,
            system_in_devices: Vec::new(),
            system_out_devices: Vec::new(),
            sample_rates: Vec::new(),
            buffer_size: BufferSizeInfo::UnknownSize,
            active: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SystemMidiDeviceInfo {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MidiServerInfo {
    pub name: String,
    pub system_in_devices: Vec<SystemMidiDeviceInfo>,
    pub system_out_devices: Vec<SystemMidiDeviceInfo>,
    pub active: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct AudioDeviceConfig {
    /// The ID to use for this device. This ID is for the "internal" device that appears to the user
    /// as list of available sources/sends. This is not necessarily the same as the name of the actual
    /// system hardware device that this "internal" device is connected to.
    ///
    /// This ID *must* be unique for each `AudioDeviceConfig` and `MidiDeviceConfig`.
    ///
    /// Examples of IDs can include:
    ///
    /// * Realtek Device In
    /// * Drums Mic
    /// * Headphones Out
    /// * Speakers Out
    pub id: String,

    /// The names of the system ports this device is connected to.
    pub system_ports: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct AudioServerConfig {
    /// The name of the audio server to use.
    pub server_name: String,

    /// The audio input devices to create/use. These devices are the "internal" devices that appears to the user
    /// as list of available sources/sends. This is not necessarily the same as the actual
    /// system hardware devices that these "internal" devices are connected to.
    ///
    /// Examples of device IDs can include:
    ///
    /// * Realtek Device In
    /// * Built-In Mic
    /// * Drums Mic
    pub use_in_devices: Vec<AudioDeviceConfig>,

    /// The audio output devices to create/use. These devices are the "internal" devices that appears to the user
    /// as list of available sources/sends. This is not necessarily the same as the actual
    /// system hardware devices that these "internal" devices are connected to.
    ///
    /// Examples of IDs can include:
    ///
    /// * Realtek Device Stereo Out
    /// * Headphones Out
    /// * Speakers Out
    pub use_out_devices: Vec<AudioDeviceConfig>,

    /// The sample rate to use.
    ///
    /// Set this to `None` to use the default sample-rate of the audio server.
    pub use_sample_rate: Option<u32>,

    /// The maximum number of frames per channel.
    ///
    /// Set this to `None` to use the default settings of the audio server.
    pub use_max_buffer_size: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct MidiDeviceConfig {
    /// The ID to use for this device. This ID is for the "internal" device that appears to the user
    /// as list of available sources/sends. This is not necessarily the same as the name of the actual
    /// system hardware device that this "internal" device is connected to.
    ///
    /// This ID *must* be unique for each `AudioDeviceConfig` and `MidiDeviceConfig`.
    pub id: String,

    /// The name of the system port this device is connected to.
    pub system_port: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct MidiServerConfig {
    /// The name of the audio server to use.
    pub server_name: String,

    /// The midi input devices to create/use. These devices are the "internal" devices that appears to the user
    /// as list of available sources/sends. This is not necessarily the same as the actual
    /// system hardware devices that these "internal" devices are connected to.
    pub use_in_devices: Vec<MidiDeviceConfig>,

    /// The midi output devices to create/use. These devices are the "internal" devices that appears to the user
    /// as list of available sources/sends. This is not necessarily the same as the actual
    /// system hardware devices that these "internal" devices are connected to.
    pub use_out_devices: Vec<MidiDeviceConfig>,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct DeviceIndex(usize);

impl DeviceIndex {
    pub(crate) fn new(index: usize) -> Self {
        Self(index)
    }

    pub fn index(&self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InternalAudioDevice {
    /// The ID of this device. This ID is for the "internal" device that appears to the user
    /// as list of available sources/sends. This is not necessarily the same as the name of the actual
    /// system hardware device that this "internal" device is connected to.
    ///
    /// This ID is unique for every `InternalAudioDevice` and `InternalMidiDevice`.
    ///
    /// Examples of IDs can include:
    ///
    /// * Realtek Device In
    /// * Drums Mic
    /// * Headphones Out
    /// * Speakers Out
    pub id_name: String,

    /// The index were this device appears in the realtime thread's buffers. This is what should actually be sent
    /// to the realtime thread for communication on what device to use.
    pub id_index: DeviceIndex,

    /// The names of the system ports this device is connected to.
    pub system_ports: Vec<String>,

    /// The number of channels in this device.
    pub channels: u16,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InternalMidiDevice {
    /// The ID of this device. This ID is for the "internal" device that appears to the user
    /// as list of available sources/sends. This is not necessarily the same as the name of the actual
    /// system hardware device that this "internal" device is connected to.
    ///
    /// This ID is unique for every `InternalAudioDevice` and `InternalMidiDevice`.
    pub id_name: String,

    /// The index were this device appears in the realtime thread's buffers. This is what should actually be sent
    /// to the realtime thread for communication on which device to use.
    pub id_index: DeviceIndex,

    /// The name of the system port this device is connected to.
    pub system_port: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StreamInfo {
    pub server_name: String,
    pub audio_in: Vec<InternalAudioDevice>,
    pub audio_out: Vec<InternalAudioDevice>,
    pub midi_in: Vec<InternalMidiDevice>,
    pub midi_out: Vec<InternalMidiDevice>,
    pub sample_rate: u32,
    pub audio_buffer_size: BufferSizeInfo,
}

pub trait RtProcessHandler: 'static + Send + Sized {
    /// Initialize/allocate any buffers here. This will only be called once
    /// on creation.
    fn init(&mut self, stream_info: &StreamInfo);

    fn process(&mut self, proc_info: ProcessInfo);
}

#[derive(Debug)]
pub struct AudioDeviceBuffer {
    pub(crate) channel_buffers: Vec<Vec<f32>>,
    pub(crate) frames: usize,
}

impl AudioDeviceBuffer {
    pub(crate) fn new(channels: u16, max_buffer_size: u32) -> AudioDeviceBuffer {
        let mut channel_buffers = Vec::<Vec<f32>>::new();
        for _ in 0..channels {
            channel_buffers.push(Vec::<f32>::with_capacity(max_buffer_size as usize));
        }

        AudioDeviceBuffer {
            channel_buffers,
            frames: 0,
        }
    }

    pub(crate) fn clear_and_resize(&mut self, frames: usize) {
        for channel in self.channel_buffers.iter_mut() {
            channel.clear();

            // This should never allocate because each buffer was given a capacity of
            // the maximum buffer size that the audio server will send.
            channel.resize(frames, 0.0);
        }

        self.frames = frames;
    }

    pub fn get(&self, channel: usize) -> Option<&[f32]> {
        self.channel_buffers.get(channel).map(|c| c.as_slice())
    }

    pub fn get_mut(&mut self, channel: usize) -> Option<&mut [f32]> {
        self.channel_buffers
            .get_mut(channel)
            .map(|c| c.as_mut_slice())
    }

    pub fn channels(&self) -> &[Vec<f32>] {
        self.channel_buffers.as_slice()
    }

    pub fn channels_mut(&mut self) -> &mut [Vec<f32>] {
        self.channel_buffers.as_mut_slice()
    }

    pub fn num_channels(&self) -> usize {
        self.channel_buffers.len()
    }

    pub fn frames(&self) -> usize {
        self.frames
    }
}

impl std::ops::Index<usize> for AudioDeviceBuffer {
    type Output = [f32];

    fn index(&self, index: usize) -> &Self::Output {
        self.channel_buffers[index].as_slice()
    }
}
impl std::ops::IndexMut<usize> for AudioDeviceBuffer {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.channel_buffers[index].as_mut_slice()
    }
}

pub struct ProcessInfo<'a> {
    pub audio_in: &'a [AudioDeviceBuffer],
    pub audio_out: &'a mut [AudioDeviceBuffer],
    pub audio_frames: usize,

    pub midi_in: &'a [MidiDeviceBuffer],
    pub midi_out: &'a mut [MidiDeviceBuffer],

    pub sample_rate: u32,
    // TODO: MIDI IO
}

#[derive(Debug)]
pub enum SpawnRtThreadError {
    AudioServerUnavailable(String),
    SystemAudioPortNotFound(String),
    NoSystemPortsGiven(String),
    DeviceIdNotUnique(String),
    PlatformSpecific(Box<dyn std::error::Error + Send + 'static>),
}

impl std::error::Error for SpawnRtThreadError {}

impl std::fmt::Display for SpawnRtThreadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpawnRtThreadError::AudioServerUnavailable(server) => {
                write!(
                    f,
                    "Error spawning rt thread: The audio sever is unavailable: {}.",
                    server
                )
            }
            SpawnRtThreadError::SystemAudioPortNotFound(port) => {
                write!(
                    f,
                    "Error spawning rt thread: The system audio port {} could not be found",
                    port,
                )
            }
            SpawnRtThreadError::NoSystemPortsGiven(id) => {
                write!(
                    f,
                    "Error spawning rt thread: No system ports were set for the device with id {}",
                    id,
                )
            }
            SpawnRtThreadError::DeviceIdNotUnique(id) => {
                write!(
                    f,
                    "Error spawning rt thread: Two or more devices have the same id {}",
                    id,
                )
            }
            SpawnRtThreadError::PlatformSpecific(e) => {
                write!(f, "Error spawning rt thread: Platform error: {}", e)
            }
        }
    }
}

#[derive(Debug)]
pub enum StreamError {
    AudioServerDisconnected(String),
    AudioDeviceDisconnected(String),
    PlatformSpecific(Box<dyn std::error::Error + Send + 'static>),
}

impl std::error::Error for StreamError {}

impl std::fmt::Display for StreamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StreamError::AudioServerDisconnected(server) => {
                write!(
                    f,
                    "Stream error: The audio sever was disconnected: {}.",
                    server
                )
            }
            StreamError::AudioDeviceDisconnected(device) => {
                write!(
                    f,
                    "Stream error: The audio device was disconnected: {}.",
                    device
                )
            }
            StreamError::PlatformSpecific(e) => {
                write!(f, "Stream error: Platform error: {}", e)
            }
        }
    }
}
