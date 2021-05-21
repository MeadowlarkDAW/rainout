use crate::{
    AudioDeviceBuffers, AudioDeviceConfig, AudioDeviceInfo, AudioDeviceStreamInfo, AudioServerInfo,
    BufferSizeInfo, MidiDeviceConfig, MidiDeviceInfo, MidiServerInfo, ProcessInfo,
    RtProcessHandler, SpawnRtThreadError, StreamError, StreamInfo,
};

static MAX_JACK_CHANNELS: u16 = 64;

pub fn refresh_audio_server(server: &mut AudioServerInfo) {
    server.devices.clear();

    match jack::Client::new(
        "rustydaw_io_dummy_client",
        jack::ClientOptions::NO_START_SERVER,
    ) {
        Ok((client, _status)) => {
            let sample_rate = client.sample_rate() as u32;
            let max_buffer_size = client.buffer_size() as u32;

            server.sample_rates = vec![sample_rate];
            server.buffer_size = BufferSizeInfo::MaximumSize(max_buffer_size);

            // Only one jack device is ever used.

            server.devices.push(AudioDeviceInfo {
                name: String::from("Jack System Audio"),

                min_output_channels: 0,
                max_output_channels: MAX_JACK_CHANNELS,

                min_input_channels: 0,
                max_input_channels: MAX_JACK_CHANNELS,
            });

            server.active = true;
        }
        Err(_) => {
            server.sample_rates.clear();
            server.buffer_size = BufferSizeInfo::UnknownSize;
            server.active = false;
        }
    }
}

pub fn refresh_midi_server(server: &mut MidiServerInfo) {
    server.in_devices.clear();
    server.out_devices.clear();

    match jack::Client::new(
        "rustydaw_io_dummy_client",
        jack::ClientOptions::NO_START_SERVER,
    ) {
        Ok((client, _status)) => {
            // Get existing midi ports.
            let system_in_ports: Vec<String> =
                client.ports(None, Some("8 bit raw midi"), jack::PortFlags::IS_OUTPUT);
            let system_out_ports: Vec<String> =
                client.ports(None, Some("8 bit raw midi"), jack::PortFlags::IS_INPUT);

            for system_in_port in system_in_ports.iter() {
                server.in_devices.push(MidiDeviceInfo {
                    name: system_in_port.clone(),
                });
            }
            for system_out_port in system_out_ports.iter() {
                server.out_devices.push(MidiDeviceInfo {
                    name: system_out_port.clone(),
                });
            }

            server.active = true;
        }
        Err(_) => {
            server.active = false;
        }
    }
}

pub struct JackRtThreadHandle<P: RtProcessHandler, E>
where
    E: 'static + Send + Sync + FnOnce(StreamError),
{
    _async_client: jack::AsyncClient<JackNotificationHandler<E>, JackProcessHandler<P>>,
}

pub fn spawn_rt_thread<P: RtProcessHandler, E>(
    audio_device_config: &AudioDeviceConfig,
    midi_in_device_config: Option<&[MidiDeviceConfig]>,
    midi_out_device_config: Option<&[MidiDeviceConfig]>,
    mut rt_process_handler: P,
    error_callback: E,
    use_client_name: Option<String>,
) -> Result<(StreamInfo, JackRtThreadHandle<P, E>), SpawnRtThreadError>
where
    E: 'static + Send + Sync + FnOnce(StreamError),
{
    if &audio_device_config.device_name != "Jack System Audio" {
        return Err(SpawnRtThreadError::AudioDeviceNotFoundInServer(
            audio_device_config.device_name.clone(),
            String::from("Jack"),
        ));
    }

    let client_name = use_client_name.unwrap_or(String::from("rusty-daw-io"));

    let (client, _status) = jack::Client::new(&client_name, jack::ClientOptions::NO_START_SERVER)?;

    let audio_out_channels = audio_device_config
        .use_num_outputs
        .unwrap_or(2)
        .min(MAX_JACK_CHANNELS);
    let audio_in_channels = audio_device_config
        .use_num_inputs
        .unwrap_or(2)
        .min(MAX_JACK_CHANNELS);

    let mut audio_in_ports = Vec::<jack::Port<jack::AudioIn>>::new();
    let mut audio_in_port_names = Vec::<String>::new();
    for i in 0..audio_in_channels {
        let name = format!("audio_in_{}", i + 1);
        let port = client.register_port(&name, jack::AudioIn::default())?;
        audio_in_port_names.push(port.name()?);
        audio_in_ports.push(port);
    }

    let mut audio_out_ports = Vec::<jack::Port<jack::AudioOut>>::new();
    let mut audio_out_port_names = Vec::<String>::new();
    for i in 0..audio_out_channels {
        let name = format!("audio_out_{}", i + 1);
        let port = client.register_port(&name, jack::AudioOut::default())?;
        audio_out_port_names.push(port.name()?);
        audio_out_ports.push(port);
    }

    let sample_rate = client.sample_rate();
    let max_buffer_size = client.buffer_size();

    // Only one Jack device is ever used.
    let stream_info = StreamInfo {
        server_name: String::from("Jack"),
        audio_devices: vec![AudioDeviceStreamInfo {
            name: String::from("Jack System Audio"),
            inputs: audio_in_channels,
            outputs: audio_out_channels,
        }],
        midi_in_devices: vec![],
        midi_out_devices: vec![],
        sample_rate: sample_rate as u32,
        audio_buffer_size: BufferSizeInfo::MaximumSize(max_buffer_size),
    };

    rt_process_handler.init(&stream_info);

    let process = JackProcessHandler::new(
        rt_process_handler,
        audio_in_ports,
        audio_out_ports,
        sample_rate as u32,
        max_buffer_size,
    );

    // Activate the client, which starts the processing.
    let async_client = client.activate_async(
        JackNotificationHandler {
            error_callback: Some(error_callback),
        },
        process,
    )?;

    // Try to automatically connect to system inputs/outputs.

    // Find system audio inputs
    let system_in_ports: Vec<String> = async_client.as_client().ports(
        None,
        Some("32 bit float mono audio"),
        jack::PortFlags::IS_PHYSICAL | jack::PortFlags::IS_OUTPUT,
    );
    // Find system audio outputs
    let system_out_ports: Vec<String> = async_client.as_client().ports(
        None,
        Some("32 bit float mono audio"),
        jack::PortFlags::IS_PHYSICAL | jack::PortFlags::IS_INPUT,
    );

    for (system_in_port, in_port) in system_in_ports.iter().zip(audio_in_port_names.iter()) {
        async_client
            .as_client()
            .connect_ports_by_name(system_in_port, in_port)?;
    }
    for (system_out_port, out_port) in system_out_ports.iter().zip(audio_out_port_names.iter()) {
        async_client
            .as_client()
            .connect_ports_by_name(out_port, system_out_port)?;
    }

    Ok((
        stream_info,
        JackRtThreadHandle {
            _async_client: async_client,
        },
    ))
}

struct JackProcessHandler<P: RtProcessHandler> {
    rt_process_handler: P,

    audio_in_ports: Vec<jack::Port<jack::AudioIn>>,
    audio_out_ports: Vec<jack::Port<jack::AudioOut>>,

    audio_device_buffers: Vec<AudioDeviceBuffers>,

    sample_rate: u32,
}

impl<P: RtProcessHandler> JackProcessHandler<P> {
    fn new(
        rt_process_handler: P,
        audio_in_ports: Vec<jack::Port<jack::AudioIn>>,
        audio_out_ports: Vec<jack::Port<jack::AudioOut>>,
        sample_rate: u32,
        max_buffer_size: u32,
    ) -> Self {
        // Only one Jack device is ever used.
        let mut audio_device_buffers = vec![AudioDeviceBuffers {
            device_name: String::from("Jack System Audio"),
            inputs: Vec::new(),
            outputs: Vec::new(),
        }];

        for _ in 0..audio_in_ports.len() {
            audio_device_buffers[0]
                .inputs
                .push(Vec::with_capacity(max_buffer_size as usize));
        }
        for _ in 0..audio_out_ports.len() {
            audio_device_buffers[0]
                .outputs
                .push(Vec::with_capacity(max_buffer_size as usize));
        }

        Self {
            rt_process_handler,
            audio_in_ports,
            audio_out_ports,
            audio_device_buffers,
            sample_rate,
        }
    }
}

impl<P: RtProcessHandler> jack::ProcessHandler for JackProcessHandler<P> {
    fn process(&mut self, _: &jack::Client, ps: &jack::ProcessScope) -> jack::Control {
        let mut audio_frames = 0;

        for (buffer, port) in self.audio_device_buffers[0]
            .inputs
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

        for buffer in self.audio_device_buffers[0].outputs.iter_mut() {
            // Clear output buffer with zeros
            buffer.clear();

            // This in theory will never actually allocate more memory because the vec
            // was preallocated with the maximum buffer size that jack will send.
            buffer.resize(audio_frames, 0.0);
        }

        self.rt_process_handler.process(ProcessInfo {
            audio_devices: &mut self.audio_device_buffers,
            audio_frames,

            sample_rate: self.sample_rate,
        });

        for (buffer, port) in self.audio_device_buffers[0]
            .outputs
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

struct JackNotificationHandler<E>
where
    E: 'static + Send + Sync + FnOnce(StreamError),
{
    error_callback: Option<E>,
}

impl<E> jack::NotificationHandler for JackNotificationHandler<E>
where
    E: 'static + Send + Sync + FnOnce(StreamError),
{
    fn thread_init(&self, _: &jack::Client) {
        println!("JACK: thread init");
    }

    fn shutdown(&mut self, status: jack::ClientStatus, reason: &str) {
        let msg = format!(
            "JACK: shutdown with status {:?} because \"{}\"",
            status, reason
        );

        println!("{:?}", msg);

        if let Some(error_callback) = self.error_callback.take() {
            (error_callback)(StreamError::AudioServerDisconnected(msg))
        }
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
