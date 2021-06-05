use crate::BufferSizeInfo;

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
pub struct AudioBus {
    /// The ID of this bus. This ID is for the "internal" bus that appears to the user
    /// as list of available sources/sends. This is not necessarily the same as the name of the actual
    /// system hardware device that this "internal" bus is connected to.
    ///
    /// This ID is unique for every `AudioBus` and `MidiController`.
    ///
    /// Examples of IDs can include:
    ///
    /// * Realtek Device In
    /// * Drums Mic
    /// * Headphones Out
    /// * Speakers Out
    pub id_name: String,

    /// The index were this bus appears in the realtime thread's buffers. This is what should actually be sent
    /// to the realtime thread for communication on what bus to use.
    pub id_index: DeviceIndex,

    /// The name of the system device this bus is connected to.
    pub system_device: String,

    /// The name of the system half duplex device this bus is connected to.
    pub system_half_duplex_device: Option<String>,

    /// The ports of the system device that are connected to this bus.
    pub system_ports: Vec<String>,

    /// The number of channels in this bus.
    pub channels: u16,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MidiController {
    /// The ID of this controller. This ID is for the "internal" controller that appears to the user
    /// as list of available sources/sends. This is not necessarily the same as the name of the actual
    /// system hardware device that this "internal" controller is connected to.
    ///
    /// This ID is unique for every `AudioBus` and `MidiController`.
    pub id_name: String,

    /// The index were this controller appears in the realtime thread's buffers. This is what should actually be sent
    /// to the realtime thread for communication on which controller to use.
    pub id_index: DeviceIndex,

    /// The name of the system port this controller is connected to.
    pub system_port: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StreamInfo {
    pub server_name: String,
    pub audio_in: Vec<AudioBus>,
    pub audio_out: Vec<AudioBus>,
    pub midi_in: Vec<MidiController>,
    pub midi_out: Vec<MidiController>,
    pub sample_rate: u32,
    pub audio_buffer_size: BufferSizeInfo,
}
