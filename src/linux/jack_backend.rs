use crate::{
    AudioDeviceAvailableConfigs, AudioDeviceConfig, AudioServerConfig, BlockSizeConfigs,
    ProcessInfo, SpawnRtThreadError,
};

static MAX_JACK_CHANNELS: u16 = 64;

pub fn refresh_audio_server(server: &mut AudioServerConfig) {
    println!("Searching for jack server...");

    match jack::Client::new(
        "rustydaw_io_dummy_client",
        jack::ClientOptions::NO_START_SERVER,
    ) {
        Ok((client, status)) => {
            let sample_rate = client.sample_rate();
            let block_size = client.buffer_size();

            println!(
                "Jack server found. Status = {:?}, samplerate = {}, block size = {}",
                status, sample_rate, block_size
            );

            let available_configs = AudioDeviceAvailableConfigs {
                sample_rates: vec![sample_rate as u32],

                min_output_channels: 0,
                max_output_channels: MAX_JACK_CHANNELS,

                min_input_channels: 0,
                max_input_channels: MAX_JACK_CHANNELS,

                block_size: BlockSizeConfigs::ConstantSize {
                    min_block_size: block_size,
                    max_block_size: block_size,
                },
            };

            // Only one jack device is ever used.

            if let Some(device) = server.devices.first_mut() {
                device.update_available_configs(available_configs);
            } else {
                server.devices.push(AudioDeviceConfig::new(
                    String::from("Default Jack Device"),
                    available_configs,
                ));
            }

            server.active = true;
        }
        Err(e) => {
            println!("Error searching for jack server: {:?}", e);

            server.active = false;
            server.devices.clear();
        }
    }
}

pub struct JackRtThreadHandle<C>
where
    C: 'static + Send + FnMut(ProcessInfo),
{
    async_client: jack::AsyncClient<JackNotificationHandler, JackProcessHandler<C>>,

    // Port names are stored in order to connect them to other ports in jack automatically
    audio_in_port_names: Vec<String>,
    audio_out_port_names: Vec<String>,
}

pub fn spawn_rt_thread<C>(
    rt_callback: C,
    audio_server_config: &AudioServerConfig,
    client_name: Option<String>,
) -> Result<JackRtThreadHandle<C>, SpawnRtThreadError>
where
    C: 'static + Send + FnMut(ProcessInfo),
{
    if let Some(audio_device_config) = audio_server_config.devices.first() {
        if audio_device_config.selected {
            let (client, _status) = jack::Client::new(
                client_name
                    .as_ref()
                    .unwrap_or(&String::from("rusty-daw-io-jack-client"))
                    .as_str(),
                jack::ClientOptions::NO_START_SERVER,
            )?;

            let audio_out_channels = audio_device_config.output_channels.unwrap_or(2);
            let audio_in_channels = audio_device_config.input_channels.unwrap_or(2);

            let mut audio_in_ports = Vec::<jack::Port<jack::AudioIn>>::new();
            let mut audio_in_port_names = Vec::<String>::new();
            let mut audio_inputs = Vec::<&[f32]>::new();
            for i in 0..audio_in_channels {
                let name = format!("audio_in_{}", i);
                audio_in_ports.push(client.register_port(&name, jack::AudioIn::default())?);
                audio_in_port_names.push(name);
                audio_inputs.push(&[]);
            }

            let mut audio_out_ports = Vec::<jack::Port<jack::AudioOut>>::new();
            let mut audio_out_port_names = Vec::<String>::new();
            let mut audio_outputs = Vec::<&mut [f32]>::new();
            for i in 0..audio_out_channels {
                let name = format!("audio_out_{}", i);
                audio_out_ports.push(client.register_port(&name, jack::AudioOut::default())?);
                audio_out_port_names.push(name);
                audio_outputs.push(&mut []);
            }

            let sample_rate = client.sample_rate();
            let max_block_size = client.buffer_size();

            let process = JackProcessHandler::new(
                rt_callback,
                audio_in_ports,
                audio_out_ports,
                sample_rate as u32,
                max_block_size,
            );

            // Activate the client, which starts the processing.
            let async_client = client.activate_async(JackNotificationHandler, process)?;

            Ok(JackRtThreadHandle {
                async_client,
                audio_in_port_names,
                audio_out_port_names,
            })
        } else {
            Err(SpawnRtThreadError::NoAudioDeviceSelected(String::from(
                audio_server_config.name(),
            )))
        }
    } else {
        Err(SpawnRtThreadError::NoAudioDeviceSelected(String::from(
            audio_server_config.name(),
        )))
    }
}

struct JackProcessHandler<C>
where
    C: 'static + Send + FnMut(ProcessInfo),
{
    callback: C,

    audio_in_ports: Vec<jack::Port<jack::AudioIn>>,
    audio_out_ports: Vec<jack::Port<jack::AudioOut>>,

    audio_in_buffers: Vec<Vec<f32>>,
    audio_out_buffers: Vec<Vec<f32>>,

    sample_rate: u32,
}

impl<C> JackProcessHandler<C>
where
    C: 'static + Send + FnMut(ProcessInfo),
{
    fn new(
        callback: C,
        audio_in_ports: Vec<jack::Port<jack::AudioIn>>,
        audio_out_ports: Vec<jack::Port<jack::AudioOut>>,
        sample_rate: u32,
        max_block_size: u32,
    ) -> Self {
        let mut audio_in_buffers = Vec::<Vec<f32>>::new();
        for _ in 0..audio_in_buffers.len() {
            audio_in_buffers.push(Vec::with_capacity(max_block_size as usize));
        }

        let mut audio_out_buffers = Vec::<Vec<f32>>::new();
        for _ in 0..audio_out_buffers.len() {
            audio_out_buffers.push(Vec::with_capacity(max_block_size as usize));
        }

        Self {
            callback,
            audio_in_ports,
            audio_out_ports,
            audio_in_buffers,
            audio_out_buffers,
            sample_rate,
        }
    }
}

impl<C> jack::ProcessHandler for JackProcessHandler<C>
where
    C: 'static + Send + FnMut(ProcessInfo),
{
    fn process(&mut self, _: &jack::Client, ps: &jack::ProcessScope) -> jack::Control {
        let mut audio_frames = 0;
        for (buffer, port) in self
            .audio_in_buffers
            .iter_mut()
            .zip(self.audio_in_ports.iter())
        {
            let in_slice = port.as_slice(ps);

            // This in theory will never actually allocate more memory because the vec
            // was preallocated with the maximum buffer size that jack will send.
            buffer.resize(in_slice.len(), 0.0);
            buffer.copy_from_slice(in_slice);

            audio_frames = in_slice.len();
        }

        if audio_frames == 0 {
            // No input channels, check output for audio_frames instead.
            if let Some(out_port) = self.audio_out_ports.first_mut() {
                audio_frames = out_port.as_mut_slice(ps).len();
            }
        }

        for buffer in self.audio_out_buffers.iter_mut() {
            // Clear output buffer with zeros
            buffer.clear();

            // This in theory will never actually allocate more memory because the vec
            // was preallocated with the maximum buffer size that jack will send.
            buffer.resize(audio_frames, 0.0);
        }

        let audio_in_channels = self.audio_in_buffers.len() as u16;
        let audio_out_channels = self.audio_out_buffers.len() as u16;

        (self.callback)(ProcessInfo {
            audio_inputs: &self.audio_in_buffers,
            audio_outputs: &mut self.audio_out_buffers,

            audio_in_channels,
            audio_out_channels,

            audio_frames,

            sample_rate: self.sample_rate,
        });

        for (buffer, port) in self
            .audio_out_buffers
            .iter()
            .zip(self.audio_out_ports.iter_mut())
        {
            let out_slice = port.as_mut_slice(ps);

            // Just in case the user for some reason resized the output buffer.
            let len = buffer.len().min(out_slice.len());

            &mut out_slice[0..len].copy_from_slice(buffer.as_slice());
        }

        jack::Control::Continue
    }
}

struct JackNotificationHandler;

impl jack::NotificationHandler for JackNotificationHandler {
    fn thread_init(&self, _: &jack::Client) {
        println!("JACK: thread init");
    }

    fn shutdown(&mut self, status: jack::ClientStatus, reason: &str) {
        println!(
            "JACK: shutdown with status {:?} because \"{}\"",
            status, reason
        );
    }

    fn freewheel(&mut self, _: &jack::Client, is_enabled: bool) {
        println!(
            "JACK: freewheel mode is {}",
            if is_enabled { "on" } else { "off" }
        );
    }

    fn sample_rate(&mut self, _: &jack::Client, srate: jack::Frames) -> jack::Control {
        println!("JACK: sample rate changed to {}", srate);
        jack::Control::Continue
    }

    fn client_registration(&mut self, _: &jack::Client, name: &str, is_reg: bool) {
        println!(
            "JACK: {} client with name \"{}\"",
            if is_reg { "registered" } else { "unregistered" },
            name
        );
    }

    fn port_registration(&mut self, _: &jack::Client, port_id: jack::PortId, is_reg: bool) {
        println!(
            "JACK: {} port with id {}",
            if is_reg { "registered" } else { "unregistered" },
            port_id
        );
    }

    fn port_rename(
        &mut self,
        _: &jack::Client,
        port_id: jack::PortId,
        old_name: &str,
        new_name: &str,
    ) -> jack::Control {
        println!(
            "JACK: port with id {} renamed from {} to {}",
            port_id, old_name, new_name
        );
        jack::Control::Continue
    }

    fn ports_connected(
        &mut self,
        _: &jack::Client,
        port_id_a: jack::PortId,
        port_id_b: jack::PortId,
        are_connected: bool,
    ) {
        println!(
            "JACK: ports with id {} and {} are {}",
            port_id_a,
            port_id_b,
            if are_connected {
                "connected"
            } else {
                "disconnected"
            }
        );
    }

    fn graph_reorder(&mut self, _: &jack::Client) -> jack::Control {
        println!("JACK: graph reordered");
        jack::Control::Continue
    }

    fn xrun(&mut self, _: &jack::Client) -> jack::Control {
        //println!("JACK: xrun occurred");
        jack::Control::Continue
    }

    fn latency(&mut self, _: &jack::Client, mode: jack::LatencyType) {
        println!(
            "JACK: {} latency has changed",
            match mode {
                jack::LatencyType::Capture => "capture",
                jack::LatencyType::Playback => "playback",
            }
        );
    }
}

impl From<jack::Error> for SpawnRtThreadError {
    fn from(e: jack::Error) -> Self {
        SpawnRtThreadError::PlatformSpecific(Box::new(e))
    }
}
