use crate::{AudioDevice, AudioServer, AudioDeviceAvailableConfigs, BlockSizeConfigs};

pub struct JackAudioServer{
    available_devices: Vec<Box<dyn AudioDevice>>,
}

impl JackAudioServer {
    pub(crate) fn new() -> Self {
        println!("Searching for jack server...");

        // Only one Jack device is ever needed, but a Vec needs to be returned anyway.
        let mut available_devices = Vec::<Box<dyn AudioDevice>>::new();

        match jack::Client::new("rustydaw_io_dummy_client", jack::ClientOptions::NO_START_SERVER) {
            Ok((client, status)) => {
                let sample_rate = client.sample_rate();
                let block_size = client.buffer_size();

                println!("Jack server found. Status = {:?}, samplerate = {}, block size = {}", status, sample_rate, block_size);

                available_devices.push(Box::new(JackAudioDevice::new(sample_rate as u32, block_size as u32)));
            }
            Err(e) => {
                println!("Error searching for jack server: {:?}", e);

                // Send back empty Vec (signals that Jack is not available)
            }
        }

        Self { available_devices }
    }

    pub(crate) fn refresh(&mut self) {
        match jack::Client::new("rustydaw_io_dummy_client", jack::ClientOptions::NO_START_SERVER) {
            Ok((client, status)) => {
                let sample_rate = client.sample_rate();
                let block_size = client.buffer_size();

                println!("Jack server found. Status = {:?}, samplerate = {}, block size = {}", status, sample_rate, block_size);

                if let Some(device) = self.available_devices.first_mut() {
                    device.downcast::<JackAudioDevice>().unwrap().refresh(sample_rate, block_size);
                } else {
                    self.available_devices.push(Box::new(JackAudioDevice::new(sample_rate as u32, block_size as u32));
                }
            }
            Err(e) => {
                println!("Error searching for jack server: {:?}", e);

                self.available_devices.clear();
            }
        }
    }
}

impl AudioServer for JackAudioServer {
    fn name(&self) -> &'static str {
        "Jack"
    }

    fn version(&self) -> Option<&str> {
        None
    }

    fn available_devices(&self) -> &Vec<Box<dyn AudioDevice>> {
        &self.available_devices
    }
}