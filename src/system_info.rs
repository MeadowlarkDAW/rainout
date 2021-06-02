#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BufferSizeInfo {
    Range { min: u32, max: u32 },
    ConstantSize(u32),
    UnknownSize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SystemDeviceInfo {
    pub name: String,
    pub in_ports: Vec<String>,
    pub out_ports: Vec<String>,
    pub sample_rates: Vec<u32>,
    pub buffer_size: BufferSizeInfo,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AudioServerInfo {
    pub name: String,
    pub version: Option<String>,
    pub devices: Vec<SystemDeviceInfo>,
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
