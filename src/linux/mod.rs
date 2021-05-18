use super::AudioServer;

pub mod jack;

pub struct DeviceConfigurator {
    servers: [Box<dyn AudioServer>; 1],
}

impl DeviceConfigurator {
    pub fn new() -> Self {
        println!("Searching for hardware devices...");

        let new_self = Self {
            servers: [
                Box::new(jack::JackAudioServer::new()),
            ]
        };

        println!("Finished searching for hardware devices");

        new_self
    }

    pub fn refresh(&mut self) {
        println!("Searching for hardware devices...");

        self.servers[0] = Box::new(jack::JackAudioServer::new());

        println!("Finished searching for hardware devices");
    }

    pub fn available_servers(&mut self) -> &mut [Box<dyn AudioServer>] {
        &mut self.servers[..]
    }
}