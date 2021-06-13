use super::{
    AudioServerInfo, Config, FatalErrorHandler, MidiServerInfo, OsDevicesInfo, OsStreamHandle,
    RtProcessHandler, SpawnRtThreadError, StreamInfo,
};

mod jack_backend;

pub struct LinuxStreamHandle<P: RtProcessHandler, E: FatalErrorHandler> {
    stream_info: StreamInfo,
    _jack_server_handle: Option<jack_backend::JackRtThreadHandle<P, E>>,
}

impl<P: RtProcessHandler, E: FatalErrorHandler> OsStreamHandle for LinuxStreamHandle<P, E> {
    type P = P;
    type E = E;

    fn stream_info(&self) -> &StreamInfo {
        &self.stream_info
    }
}

#[derive(Debug)]
pub struct LinuxDevicesInfo {
    audio_servers_info: [AudioServerInfo; 1],
    midi_servers_info: [MidiServerInfo; 1],
}

impl Default for LinuxDevicesInfo {
    fn default() -> Self {
        let mut new_self = Self {
            audio_servers_info: [
                AudioServerInfo::new(String::from("Jack"), None), // TODO: Get Jack version?
            ],
            midi_servers_info: [
                MidiServerInfo::new(String::from("Jack"), None), // TODO: Get Jack version?
            ],
        };

        new_self.refresh_audio_servers();
        new_self.refresh_midi_servers();

        new_self
    }
}

impl OsDevicesInfo for LinuxDevicesInfo {
    fn refresh_audio_servers(&mut self) {
        // First server is Jack
        jack_backend::refresh_audio_server(&mut self.audio_servers_info[0]);
    }

    fn refresh_midi_servers(&mut self) {
        // First server is Jack
        jack_backend::refresh_midi_server(&mut self.midi_servers_info[0]);
    }

    fn audio_servers_info(&self) -> &[AudioServerInfo] {
        &self.audio_servers_info
    }
    fn midi_servers_info(&self) -> &[MidiServerInfo] {
        &self.midi_servers_info
    }

    fn default_audio_server(&self) -> &String {
        // Only Jack server for now.
        &self.audio_servers_info[0].name
    }
    fn default_midi_server(&self) -> &String {
        // Only Jack server for now.
        &self.midi_servers_info[0].name
    }

    fn estimated_latency(&self, config: &Config) -> Option<u32> {
        match config.audio_server.as_str() {
            "Jack" => {
                // First server is Jack.
                // Jack only ever uses one device.
                // Buffer size in Jack is always constant.
                if let Some(device) = self.audio_servers_info[0].devices.first() {
                    return Some(device.default_buffer_size);
                }
            }
            _ => {}
        }

        None
    }

    fn sample_rate(&self, config: &Config) -> Option<u32> {
        match config.audio_server.as_str() {
            "Jack" => {
                // First server is Jack.
                // Jack only ever uses one device.
                // Only one sample rate is available, which is the sample rate of the running Jack server.
                if let Some(device) = self.audio_servers_info[0].devices.first() {
                    return Some(device.sample_rates[0]);
                }
            }
            _ => {}
        }

        None
    }
}

pub fn spawn_rt_thread<P: RtProcessHandler, E: FatalErrorHandler>(
    config: &Config,
    use_client_name: Option<String>,
    rt_process_handler: P,
    fatal_error_handler: E,
) -> Result<LinuxStreamHandle<P, E>, SpawnRtThreadError> {
    match config.audio_server.as_str() {
        "Jack" => {
            let (stream_info, jack_server_handle) = jack_backend::spawn_rt_thread(
                config,
                rt_process_handler,
                fatal_error_handler,
                use_client_name,
            )?;

            return Ok(LinuxStreamHandle {
                stream_info,
                _jack_server_handle: Some(jack_server_handle),
            });
        }
        s => {
            let s = String::from(s);
            Err(SpawnRtThreadError::AudioServerUnavailable(s))
        }
    }
}
