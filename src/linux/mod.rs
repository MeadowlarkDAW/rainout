use super::{
    AudioServerConfig, AudioServerInfo, MidiServerConfig, MidiServerInfo, OsDevicesInfo,
    OsStreamHandle, RtProcessHandler, SpawnRtThreadError, StreamError, StreamInfo,
};

mod alsa_backend;
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
    audio_servers_info: [AudioServerInfo; 2],
    midi_servers_info: [MidiServerInfo; 2],
}

impl Default for LinuxDevicesInfo {
    fn default() -> Self {
        let mut new_self = Self {
            audio_servers_info: [
                AudioServerInfo::new(String::from("ALSA"), None), // TODO: Get ALSA version?
                AudioServerInfo::new(String::from("Jack"), None), // TODO: Get Jack version?
            ],
            midi_servers_info: [
                MidiServerInfo::new(String::from("ALSA"), None), // TODO: Get ALSA version?
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
        // First server is ALSA
        alsa_backend::refresh_audio_server(&mut self.audio_servers_info[0]);
        // Second server is Jack
        jack_backend::refresh_audio_server(&mut self.audio_servers_info[1]);
    }

    fn refresh_midi_servers(&mut self) {
        // Second server is Jack
        jack_backend::refresh_midi_server(&mut self.midi_servers_info[1]);
    }

    fn audio_servers_info(&self) -> &[AudioServerInfo] {
        &self.audio_servers_info
    }

    fn midi_servers_info(&self) -> &[MidiServerInfo] {
        &self.midi_servers_info
    }
}

pub fn spawn_rt_thread<P: RtProcessHandler, E>(
    audio_config: &AudioServerConfig,
    midi_config: Option<&MidiServerConfig>,
    use_client_name: Option<String>,
    rt_process_handler: P,
    error_callback: E,
) -> Result<LinuxStreamHandle<P, E>, SpawnRtThreadError>
where
    E: 'static + Send + Sync + FnOnce(StreamError),
{
    let audio_server_name = audio_config.server.get_name_or("Jack");
    let midi_server_name = midi_config
        .map(|m| m.server.get_name_or("Jack"))
        .unwrap_or("");

    match audio_server_name {
        "Jack" => {
            if let Some(midi_config) = midi_config {
                if midi_server_name == "Jack" {
                    let (stream_info, jack_server_handle) = jack_backend::spawn_rt_thread(
                        audio_config,
                        &midi_config.create_in_devices,
                        &midi_config.create_out_devices,
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
