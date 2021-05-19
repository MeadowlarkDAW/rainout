use super::{AudioServerConfig, RtProcessHandler, SpawnRtThreadError, StreamError};

mod jack_backend;

pub struct StreamHandle<P: RtProcessHandler, E>
where
    E: 'static + Send + Sync + FnOnce(StreamError),
{
    device_configurator: DeviceConfigurator,
    _jack_server_handle: Option<jack_backend::JackRtThreadHandle<P, E>>,
}

impl<P: RtProcessHandler, E> StreamHandle<P, E>
where
    E: 'static + Send + Sync + FnOnce(StreamError),
{
    pub fn end_stream(self) -> DeviceConfigurator {
        self.device_configurator
        // Drop handle here. This should automatically close all streams.
    }
}

pub struct DeviceConfigurator {
    server_configs: [AudioServerConfig; 1],
    client_name: Option<String>,
}

impl DeviceConfigurator {
    pub fn new(client_name: Option<String>) -> Self {
        let mut new_self = Self {
            server_configs: [
                AudioServerConfig::new(String::from("Jack"), None), // TODO: Get Jack version?
            ],
            client_name,
        };

        new_self.refresh_audio_servers();

        new_self
    }

    pub fn refresh_audio_servers(&mut self) {
        println!("Searching for audio servers...");

        // First server is Jack
        jack_backend::refresh_audio_server(&mut self.server_configs[0]);

        println!("Finished searching for audio servers");
    }

    pub fn server_configs(&self) -> &[AudioServerConfig] {
        &self.server_configs
    }
    pub fn server_configs_mut(&mut self) -> &mut [AudioServerConfig] {
        &mut self.server_configs
    }

    pub fn spawn_rt_thread<P: RtProcessHandler, E>(
        self,
        rt_process_handler: P,
        error_callback: E,
    ) -> Result<StreamHandle<P, E>, (Self, SpawnRtThreadError)>
    where
        E: 'static + Send + Sync + FnOnce(StreamError),
    {
        // First server is Jack
        {
            let jack_server_config = &self.server_configs[0];

            if jack_server_config.selected() {
                let jack_server_handle = match jack_backend::spawn_rt_thread(
                    rt_process_handler,
                    error_callback,
                    jack_server_config,
                    self.client_name.clone(),
                ) {
                    Ok(j) => j,
                    Err(e) => {
                        return Err((self, e));
                    }
                };

                return Ok(StreamHandle {
                    _jack_server_handle: Some(jack_server_handle),
                    device_configurator: self,
                });
            } else {
                // TODO: Don't return error when more servers are implemented.

                return Err((self, SpawnRtThreadError::NoAudioServerSelected));
            }
        }
    }
}
