use super::{
    AudioConfig, AudioServerDevices, AudioServerInfo, BufferSizeInfo, MidiConfig, MidiServerInfo,
    OsDevicesInfo, OsStreamHandle, RtProcessHandler, SpawnRtThreadError, StreamError, StreamInfo,
};

mod jack_backend;

pub struct LinuxStreamHandle<P: RtProcessHandler, E>
where
    E: 'static + Send + Sync + FnOnce(StreamError),
{
    stream_info: StreamInfo,
    _jack_server_handle: Option<jack_backend::JackRtThreadHandle<P, E>>,
}

impl<P: RtProcessHandler, E> OsStreamHandle for LinuxStreamHandle<P, E>
where
    E: 'static + Send + Sync + FnOnce(StreamError),
{
    type P = P;
    type E = E;

    fn stream_info(&self) -> &StreamInfo {
        &self.stream_info
    }
}

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

    fn default_audio_server(&self) -> String {
        if self.audio_servers_info[0].available {
            String::from("ALSA")
        } else {
            String::from("Jack")
        }
    }
    fn default_midi_config(&self) -> String {
        if self.midi_servers_info[0].available {
            String::from("ALSA")
        } else {
            String::from("Jack")
        }
    }

    fn estimated_latency(&self, audio_config: &AudioConfig) -> Option<u32> {
        match audio_config.server.as_str() {
            "Jack" => {
                if self.audio_servers_info[0].available {
                    // First server is Jack.
                    // Jack only ever uses one device.
                    // Buffer size in Jack is always constant.
                    if let Some(AudioServerDevices::SingleDevice(device)) =
                        &self.audio_servers_info[0].devices
                    {
                        if let BufferSizeInfo::ConstantSize(size) = &device.buffer_size {
                            return Some(*size);
                        }
                    }
                }
            }
            _ => {}
        }

        None
    }

    fn sample_rate(&self, audio_config: &AudioConfig) -> Option<u32> {
        match audio_config.server.as_str() {
            "Jack" => {
                if self.audio_servers_info[0].available {
                    // First server is Jack.
                    // Jack only ever uses one device.
                    // Only one sample rate is available, which is the sample rate of the running Jack server.
                    if let Some(AudioServerDevices::SingleDevice(device)) =
                        &self.audio_servers_info[0].devices
                    {
                        return Some(device.sample_rates[0]);
                    }
                }
            }
            _ => {}
        }

        None
    }
}

pub fn spawn_rt_thread<P: RtProcessHandler, E>(
    audio_config: &AudioConfig,
    midi_config: Option<&MidiConfig>,
    use_client_name: Option<String>,
    rt_process_handler: P,
    error_callback: E,
) -> Result<LinuxStreamHandle<P, E>, SpawnRtThreadError>
where
    E: 'static + Send + Sync + FnOnce(StreamError),
{
    let midi_server_name = midi_config.map(|m| m.server.as_str()).unwrap_or("");

    match audio_config.server.as_str() {
        "Jack" => {
            if let Some(midi_config) = midi_config {
                if midi_server_name == "Jack" {
                    let (stream_info, jack_server_handle) = jack_backend::spawn_rt_thread(
                        audio_config,
                        &midi_config.in_controllers,
                        &midi_config.out_controllers,
                        rt_process_handler,
                        error_callback,
                        use_client_name,
                    )?;

                    return Ok(LinuxStreamHandle {
                        stream_info,
                        _jack_server_handle: Some(jack_server_handle),
                    });
                }
            }

            let (stream_info, jack_server_handle) = jack_backend::spawn_rt_thread(
                audio_config,
                &[],
                &[],
                rt_process_handler,
                error_callback,
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
