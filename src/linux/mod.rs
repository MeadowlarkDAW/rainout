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
    midi_server_info: MidiServerInfo,
}

impl DeviceInfo {
    pub fn new() -> Self {
        let mut new_self = Self {
            audio_servers_info: [
                AudioServerInfo::new(String::from("Jack"), None), // TODO: Get Jack version?
            ],
            midi_server_info: MidiServerInfo {
                name: String::from("Jack"),
                system_in_devices: Vec::new(),
                system_out_devices: Vec::new(),
            },
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
        // Using midir for midi.

        self.midi_server_info.system_in_devices.clear();
        self.midi_server_info.system_out_devices.clear();
    }

    pub fn audio_servers_info(&self) -> &[AudioServerInfo] {
        &self.audio_servers_info
    }

    pub fn midi_server_info(&self) -> &MidiServerInfo {
        &self.midi_server_info
    }
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
    check_duplicate_ids(audio_config, midi_config)?;

    match audio_config.server_name.as_str() {
        "Jack" => {
            if let Some(midi_config) = midi_config {
                if midi_config.server_name == "Jack" {
                    let (stream_info, jack_server_handle) = jack_backend::spawn_rt_thread(
                        &audio_config.use_in_devices,
                        &audio_config.use_out_devices,
                        &midi_config.use_in_devices,
                        &midi_config.use_out_devices,
                        rt_process_handler,
                        error_callback,
                        use_client_name,
                    )?;

                    return Ok(StreamHandle {
                        stream_info,
                        _jack_server_handle: Some(jack_server_handle),
                    });
                }
            }

            let (stream_info, jack_server_handle) = jack_backend::spawn_rt_thread(
                &audio_config.use_in_devices,
                &audio_config.use_out_devices,
                &[],
                &[],
                rt_process_handler,
                error_callback,
                use_client_name,
            )?;

            return Ok(StreamHandle {
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

pub fn check_duplicate_ids(
    audio_config: &AudioServerConfig,
    midi_config: Option<&MidiServerConfig>,
) -> Result<(), SpawnRtThreadError> {
    let mut device_ids = std::collections::HashSet::new();

    for in_device in audio_config.use_in_devices.iter() {
        if !device_ids.insert(in_device.id.clone()) {
            return Err(SpawnRtThreadError::DeviceIdNotUnique(in_device.id.clone()));
        }
    }
    for out_device in audio_config.use_out_devices.iter() {
        if !device_ids.insert(out_device.id.clone()) {
            return Err(SpawnRtThreadError::DeviceIdNotUnique(out_device.id.clone()));
        }
    }

    if let Some(midi_config) = midi_config {
        for in_device in midi_config.use_in_devices.iter() {
            if !device_ids.insert(in_device.id.clone()) {
                return Err(SpawnRtThreadError::DeviceIdNotUnique(in_device.id.clone()));
            }
        }
        for out_device in midi_config.use_out_devices.iter() {
            if !device_ids.insert(out_device.id.clone()) {
                return Err(SpawnRtThreadError::DeviceIdNotUnique(out_device.id.clone()));
            }
        }
    }

    Ok(())
}
