#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BufferSizeInfo {
    Range { min: u32, max: u32 },
    MaximumSize(u32),
    ConstantSize(u32),
    UnknownSize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HalfDuplexDeviceInfo {
    pub name: String,
    pub ports: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DuplexDeviceType {
    SingleDevice {
        in_ports: Vec<String>,
        out_ports: Vec<String>,
    },
    MultiDevice {
        in_devices: Vec<HalfDuplexDeviceInfo>,
        out_devices: Vec<HalfDuplexDeviceInfo>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct DuplexDeviceInfo {
    pub name: String,
    pub devices: DuplexDeviceType,
    pub sample_rates: Vec<u32>,
    pub buffer_size: BufferSizeInfo,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AudioServerInfo {
    pub name: String,
    pub version: Option<String>,
    pub devices: Vec<DuplexDeviceInfo>,
    pub available: bool,
}

impl AudioServerInfo {
    pub(crate) fn new(name: String, version: Option<String>) -> Self {
        Self {
            name,
            version,
            devices: Vec::new(),
            available: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MidiDeviceInfo {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MidiServerInfo {
    pub name: String,
    pub version: Option<String>,
    pub in_devices: Vec<MidiDeviceInfo>,
    pub out_devices: Vec<MidiDeviceInfo>,
    pub available: bool,
}

impl MidiServerInfo {
    pub(crate) fn new(name: String, version: Option<String>) -> Self {
        Self {
            name,
            version,
            in_devices: Vec::new(),
            out_devices: Vec::new(),
            available: false,
        }
    }
}
