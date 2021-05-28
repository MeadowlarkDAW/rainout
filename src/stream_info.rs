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
