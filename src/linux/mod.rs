use super::{AudioServerConfig, ProcessInfo, SpawnRtThreadError};

mod jack_backend;

pub struct DeviceConfigurator<C>
where
    C: 'static + Send + FnMut(ProcessInfo),
{
    server_configs: [AudioServerConfig; 1],
    client_name: Option<String>,

    jack_server_handle: Option<jack_backend::JackRtThreadHandle<C>>,
}

impl<C> DeviceConfigurator<C>
where
    C: 'static + Send + FnMut(ProcessInfo),
{
    pub fn new(client_name: Option<String>) -> Self {
        let mut new_self = Self {
            server_configs: [
                AudioServerConfig::new(String::from("Jack"), None), // TODO: Get Jack version?
            ],
            client_name,

            jack_server_handle: None,
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

    pub fn spawn_rt_thread(&mut self, rt_callback: C) -> Result<(), SpawnRtThreadError> {
        // First server is Jack
        {
            let jack_server_config = &self.server_configs[0];

            if jack_server_config.selected() {
                let jack_server_handle = jack_backend::spawn_rt_thread(
                    rt_callback,
                    jack_server_config,
                    self.client_name.clone(),
                )?;
            } else {
                // TODO: Don't return error when more servers are implemented.

                return Err(SpawnRtThreadError::NoAudioServerSelected);
            }
        }

        Ok(())
    }
}
