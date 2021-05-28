#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BufferSizeInfo {
    MaximumSize(u32),
    ConstantSize(u32),
    UnknownSize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SystemAudioDeviceInfo {
    pub name: String,
    pub channels: u16,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AudioServerInfo {
    pub name: String,
    pub version: Option<String>,
    pub system_in_devices: Vec<SystemAudioDeviceInfo>,
    pub system_out_devices: Vec<SystemAudioDeviceInfo>,
    pub sample_rates: Vec<u32>,
    pub buffer_size: BufferSizeInfo,
    pub available: bool,
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
            available: false,
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
    pub version: Option<String>,
    pub system_in_devices: Vec<SystemMidiDeviceInfo>,
    pub system_out_devices: Vec<SystemMidiDeviceInfo>,
    pub available: bool,
}

impl MidiServerInfo {
    pub(crate) fn new(name: String, version: Option<String>) -> Self {
        Self {
            name,
            version,
            system_in_devices: Vec::new(),
            system_out_devices: Vec::new(),
            available: false,
        }
    }
}
