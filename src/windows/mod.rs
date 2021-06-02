use std::marker::PhantomData;

use super::{
    AudioServerConfig, AudioServerInfo, BufferSizeInfo, MidiServerConfig, MidiServerInfo,
    OsDevicesInfo, OsStreamHandle, RtProcessHandler, SpawnRtThreadError, StreamError, StreamInfo,
};

pub struct WindowsStreamHandle<P: RtProcessHandler, E>
where
    E: 'static + Send + Sync + FnOnce(StreamError),
{
    stream_info: StreamInfo,
    _phantom_p: PhantomData<P>,
    _phantom_e: PhantomData<E>,
}

impl<P: RtProcessHandler, E> OsStreamHandle for WindowsStreamHandle<P, E>
where
    E: 'static + Send + Sync + FnOnce(StreamError),
{
    type P = P;
    type E = E;

    fn stream_info(&self) -> &StreamInfo {
        &self.stream_info
    }
}

pub struct WindowsDevicesInfo {
    audio_servers_info: [AudioServerInfo; 1],
    midi_servers_info: [MidiServerInfo; 1],
}

impl Default for WindowsDevicesInfo {
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

impl OsDevicesInfo for WindowsDevicesInfo {
    fn refresh_audio_servers(&mut self) {
        // First server is Jack
        //jack_backend::refresh_audio_server(&mut self.audio_servers_info[0]);
    }

    fn refresh_midi_servers(&mut self) {
        // First server is Jack
        //jack_backend::refresh_midi_server(&mut self.midi_servers_info[0]);
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

    fn estimated_latency(&self, audio_config: &AudioServerConfig) -> u32 {
        match audio_config.server.as_str() {
            "Jack" => {
                if self.audio_servers_info[0].available {
                    // First server is Jack.
                    // Jack only ever uses one "duplex device".
                    // Buffer size in Jack is always constant.
                    if let BufferSizeInfo::ConstantSize(size) =
                        &self.audio_servers_info[0].devices[0].buffer_size
                    {
                        return *size;
                    }
                }
            }
            _ => {}
        }

        0
    }

    fn sample_rate(&self, audio_config: &AudioServerConfig) -> u32 {
        match audio_config.server.as_str() {
            "Jack" => {
                if self.audio_servers_info[0].available {
                    // First server is Jack.
                    // Jack only ever uses one "duplex device".
                    // Only one sample rate is available, which is the sample rate of the running Jack server.
                    return self.audio_servers_info[0].devices[0].sample_rates[0];
                }
            }
            _ => {}
        }

        1
    }
}

pub fn spawn_rt_thread<P: RtProcessHandler, E>(
    audio_config: &AudioServerConfig,
    midi_config: Option<&MidiServerConfig>,
    use_client_name: Option<String>,
    rt_process_handler: P,
    error_callback: E,
) -> Result<WindowsStreamHandle<P, E>, SpawnRtThreadError>
where
    E: 'static + Send + Sync + FnOnce(StreamError),
{
    Err(SpawnRtThreadError::AudioServerUnavailable(String::from("")))
}
