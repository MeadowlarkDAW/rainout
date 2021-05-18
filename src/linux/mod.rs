use super::{AudioServerConfig, ProcessInfo, SpawnRtThreadError};

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


        println!("Finished searching for audio servers");
    }

    pub fn server_configs(&self) -> &[AudioServerConfig] {
        &self.server_configs
    }
    pub fn server_configs_mut(&mut self) -> &mut [AudioServerConfig] {
        &mut self.server_configs
    }

    pub fn spawn_rt_thread<C>(&mut self, rt_callback: C) -> Result<(), SpawnRtThreadError>
    where
        C: 'static + FnMut(&ProcessInfo)
    {

        Ok(())
    }
}