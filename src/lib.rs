#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
use linux::{LinuxDevicesInfo, LinuxStreamHandle};

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
use windows::{WindowsDevicesInfo, WindowsStreamHandle};

pub mod audio_buffer;
pub mod config;
pub mod config_helper;
pub mod error;
pub mod midi_buffer;
pub mod stream_info;
pub mod system_info;

pub use audio_buffer::*;
pub use config::*;
pub use config_helper::*;
pub use midi_buffer::*;
pub use stream_info::*;
pub use system_info::*;

use error::*;

pub trait RtProcessHandler: 'static + Send {
    /// Initialize/allocate any buffers here. This will only be called once
    /// on creation.
    fn init(&mut self, stream_info: &StreamInfo);

    fn process(&mut self, proc_info: ProcessInfo);
}

pub struct ProcessInfo<'a> {
    pub audio_in: &'a [AudioDeviceBuffer],
    pub audio_out: &'a mut [AudioDeviceBuffer],
    pub audio_frames: usize,

    pub midi_in: &'a [MidiDeviceBuffer],
    pub midi_out: &'a mut [MidiDeviceBuffer],

    pub sample_rate: u32,
}

pub struct StreamHandle<P, E>
where
    P: RtProcessHandler,
    E: 'static + Send + Sync + FnOnce(StreamError),
{
    #[cfg(target_os = "linux")]
    os_handle: LinuxStreamHandle<P, E>,

    #[cfg(target_os = "windows")]
    os_handle: WindowsStreamHandle<P, E>,
}

impl<P, E> StreamHandle<P, E>
where
    P: RtProcessHandler,
    E: 'static + Send + Sync + FnOnce(StreamError),
{
    pub fn stream_info(&self) -> &StreamInfo {
        self.os_handle.stream_info()
    }
}

pub struct DevicesInfo {
    #[cfg(target_os = "linux")]
    os_info: LinuxDevicesInfo,

    #[cfg(target_os = "windows")]
    os_info: WindowsDevicesInfo,
}

impl DevicesInfo {
    pub fn new() -> Self {
        Self {
            os_info: Default::default(),
        }
    }

    pub fn refresh_audio_servers(&mut self) {
        self.os_info.refresh_audio_servers();
    }
    pub fn refresh_midi_servers(&mut self) {
        self.os_info.refresh_midi_servers();
    }

    pub fn audio_servers_info(&self) -> &[AudioServerInfo] {
        self.os_info.audio_servers_info()
    }
    pub fn midi_servers_info(&self) -> &[MidiServerInfo] {
        self.os_info.midi_servers_info()
    }

    pub fn estimated_latency(&self, audio_config: &AudioServerConfig) -> u32 {
        self.os_info.estimated_latency(audio_config)
    }
    pub fn sample_rate(&self, audio_config: &AudioServerConfig) -> u32 {
        self.os_info.sample_rate(audio_config)
    }
}

trait OsStreamHandle {
    type P: RtProcessHandler;
    type E: 'static + Send + Sync + FnOnce(StreamError);

    fn stream_info(&self) -> &StreamInfo;
}

trait OsDevicesInfo {
    fn refresh_audio_servers(&mut self);
    fn refresh_midi_servers(&mut self);

    fn audio_servers_info(&self) -> &[AudioServerInfo];
    fn midi_servers_info(&self) -> &[MidiServerInfo];

    fn default_audio_server(&self) -> String;
    fn default_midi_config(&self) -> String;

    fn estimated_latency(&self, audio_config: &AudioServerConfig) -> u32;
    fn sample_rate(&self, audio_config: &AudioServerConfig) -> u32;
}

pub fn spawn_rt_thread<P, E>(
    audio_config: &AudioServerConfig,
    midi_config: Option<&MidiServerConfig>,
    use_client_name: Option<String>,
    rt_process_handler: P,
    error_callback: E,
) -> Result<StreamHandle<P, E>, SpawnRtThreadError>
where
    P: RtProcessHandler,
    E: 'static + Send + Sync + FnOnce(StreamError),
{
    check_duplicate_ids(audio_config, midi_config)?;

    #[cfg(target_os = "linux")]
    {
        Ok(StreamHandle {
            os_handle: linux::spawn_rt_thread(
                audio_config,
                midi_config,
                use_client_name,
                rt_process_handler,
                error_callback,
            )?,
        })
    }

    #[cfg(target_os = "windows")]
    {
        Ok(StreamHandle {
            os_handle: windows::spawn_rt_thread(
                audio_config,
                midi_config,
                use_client_name,
                rt_process_handler,
                error_callback,
            )?,
        })
    }
}

fn check_duplicate_ids(
    audio_config: &AudioServerConfig,
    midi_config: Option<&MidiServerConfig>,
) -> Result<(), SpawnRtThreadError> {
    let mut device_ids = std::collections::HashSet::new();

    for in_device in audio_config.create_in_devices.iter() {
        if !device_ids.insert(in_device.id.clone()) {
            return Err(SpawnRtThreadError::DeviceIdNotUnique(in_device.id.clone()));
        }
    }
    for out_device in audio_config.create_out_devices.iter() {
        if !device_ids.insert(out_device.id.clone()) {
            return Err(SpawnRtThreadError::DeviceIdNotUnique(out_device.id.clone()));
        }
    }

    if let Some(midi_config) = midi_config {
        for in_device in midi_config.create_in_devices.iter() {
            if !device_ids.insert(in_device.id.clone()) {
                return Err(SpawnRtThreadError::DeviceIdNotUnique(in_device.id.clone()));
            }
        }
        for out_device in midi_config.create_out_devices.iter() {
            if !device_ids.insert(out_device.id.clone()) {
                return Err(SpawnRtThreadError::DeviceIdNotUnique(out_device.id.clone()));
            }
        }
    }

    Ok(())
}
