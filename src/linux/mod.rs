use super::{
    AudioServerConfig, AudioServerInfo, MidiServerConfig, MidiServerInfo, RtProcessHandler,
    SpawnRtThreadError, StreamError, StreamInfo,
};

mod jack_backend;

pub struct StreamHandle<P: RtProcessHandler, E>
where
    E: 'static + Send + Sync + FnOnce(StreamError),
{
    stream_info: StreamInfo,
    _jack_server_handle: Option<jack_backend::JackRtThreadHandle<P, E>>,
}

impl<P: RtProcessHandler, E> StreamHandle<P, E>
where
    E: 'static + Send + Sync + FnOnce(StreamError),
{
    pub fn stream_info(&self) -> &StreamInfo {
        &self.stream_info
    }
}

pub struct DeviceInfo {
    audio_servers_info: [AudioServerInfo; 1],
    midi_servers_info: [MidiServerInfo; 1],
}

impl DeviceInfo {
    pub fn new() -> Self {
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

    pub fn refresh_audio_servers(&mut self) {
        // First server is Jack
        jack_backend::refresh_audio_server(&mut self.audio_servers_info[0]);
    }

    pub fn refresh_midi_servers(&mut self) {
        // First server is Jack
        jack_backend::refresh_midi_server(&mut self.midi_servers_info[0]);
    }

    pub fn audio_server_info(&self) -> &[AudioServerInfo] {
        &self.audio_servers_info
    }
    pub fn midi_server_info(&self) -> &[MidiServerInfo] {
        &self.midi_servers_info
    }

    /*
    pub fn estimated_latency(&self) -> Option<EstimatedLatency> {
        if let Some(selected) = &self.selected_audio_server {
            match selected.as_str() {
                "Jack" => {
                    // First server is Jack
                    if let Some(jack_device) = self.audio_server_configs[0].audio_devices().first() {
                        if let BufferSizeConfigs::ConstantSize {
                            max_buffer_size, ..
                        } = jack_device.available_configs.buffer_size
                        {
                            if let Some(sample_rate) = jack_device.available_configs.sample_rates.first() {
                                return Some(EstimatedLatency {
                                    frames: max_buffer_size,
                                    sample_rate: *sample_rate,
                                });
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        None
    }
    */
}

pub fn spawn_rt_thread<P: RtProcessHandler, E>(
    audio_config: &AudioServerConfig,
    midi_config: Option<&MidiServerConfig>,
    use_client_name: Option<String>,
    rt_process_handler: P,
    error_callback: E,
) -> Result<StreamHandle<P, E>, SpawnRtThreadError>
where
    E: 'static + Send + Sync + FnOnce(StreamError),
{
    match audio_config.server_name.as_str() {
        "Jack" => {
            if let Some(device_config) = audio_config.use_devices.first() {
                let (stream_info, jack_server_handle) = jack_backend::spawn_rt_thread(
                    &device_config,
                    None,
                    None,
                    rt_process_handler,
                    error_callback,
                    use_client_name,
                )?;

                return Ok(StreamHandle {
                    stream_info,
                    _jack_server_handle: Some(jack_server_handle),
                });
            } else {
                return Err(SpawnRtThreadError::NoAudioDeviceSelected(String::from(
                    "Jack",
                )));
            }
        }
        s => {
            let s = String::from(s);
            Err(SpawnRtThreadError::AudioServerUnavailable(s))
        }
    }
}
