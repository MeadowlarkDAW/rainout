use crate::{
    AudioDeviceBuffer, AudioDeviceConfig, AudioDeviceInfo, AudioDeviceStreamInfo, AudioServerInfo,
    BufferSizeInfo, MidiDeviceConfig, MidiDeviceInfo, MidiDeviceStreamInfo, MidiServerInfo,
    ProcessInfo, RtProcessHandler, SpawnRtThreadError, StreamError, StreamInfo,
};

pub fn refresh_audio_server(server: &mut AudioServerInfo) {
    server.in_devices.clear();
    server.out_devices.clear();

    match jack::Client::new(
        "rustydaw_io_dummy_client",
        jack::ClientOptions::NO_START_SERVER,
    ) {
        Ok((client, _status)) => {
            let sample_rate = client.sample_rate() as u32;
            let max_buffer_size = client.buffer_size() as u32;

            server.sample_rates = vec![sample_rate];
            server.buffer_size = BufferSizeInfo::MaximumSize(max_buffer_size);

            let system_audio_in_ports: Vec<String> = client.ports(
                None,
                Some("32 bit float mono audio"),
                jack::PortFlags::IS_OUTPUT,
            );
            let system_audio_out_ports: Vec<String> = client.ports(
                None,
                Some("32 bit float mono audio"),
                jack::PortFlags::IS_INPUT,
            );

            for system_audio_in_port in system_audio_in_ports.iter() {
                server.in_devices.push(AudioDeviceInfo {
                    name: system_audio_in_port.clone(),

                    min_channels: 1,
                    max_channels: 1,
                });
            }
            for system_audio_out_port in system_audio_out_ports.iter() {
                server.out_devices.push(AudioDeviceInfo {
                    name: system_audio_out_port.clone(),

                    min_channels: 1,
                    max_channels: 1,
                });
            }

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
    audio_in_device_config: &[AudioDeviceConfig],
    audio_out_device_config: &[AudioDeviceConfig],
    midi_in_device_config: &[MidiDeviceConfig],
    midi_out_device_config: &[MidiDeviceConfig],
    mut rt_process_handler: P,
    error_callback: E,
    use_client_name: Option<String>,
) -> Result<(StreamInfo, JackRtThreadHandle<P, E>), SpawnRtThreadError>
where
    E: 'static + Send + Sync + FnOnce(StreamError),
{
    let client_name = use_client_name.unwrap_or(String::from("rusty-daw-io"));

    let (client, _status) = jack::Client::new(&client_name, jack::ClientOptions::NO_START_SERVER)?;

    let mut audio_in_ports = Vec::<jack::Port<jack::AudioIn>>::new();
    let mut audio_in_port_names = Vec::<String>::new();
    let mut audio_in_stream_info = Vec::<AudioDeviceStreamInfo>::new();
    let mut audio_in_port_connections = Vec::<String>::new();
    for audio_device in audio_in_device_config.iter() {
        let port = client.register_port(&audio_device.device_name, jack::AudioIn::default())?;
        audio_in_port_names.push(port.name()?);
        audio_in_ports.push(port);
        audio_in_stream_info.push(AudioDeviceStreamInfo {
            name: audio_device.device_name.clone(),
            channels: 1,
        });
        audio_in_port_connections.push(audio_device.device_name.clone());
    }

    let mut audio_out_ports = Vec::<jack::Port<jack::AudioOut>>::new();
    let mut audio_out_port_names = Vec::<String>::new();
    let mut audio_out_stream_info = Vec::<AudioDeviceStreamInfo>::new();
    let mut audio_out_port_connections = Vec::<String>::new();
    for audio_device in audio_out_device_config.iter() {
        let port = client.register_port(&audio_device.device_name, jack::AudioOut::default())?;
        audio_out_port_names.push(port.name()?);
        audio_out_ports.push(port);
        audio_out_stream_info.push(AudioDeviceStreamInfo {
            name: audio_device.device_name.clone(),
            channels: 1,
        });
        audio_out_port_connections.push(audio_device.device_name.clone());
    }

    let sample_rate = client.sample_rate();
    let max_buffer_size = client.buffer_size();

    let mut midi_in_ports = Vec::<jack::Port<jack::MidiIn>>::new();
    let mut midi_in_stream_info = Vec::<MidiDeviceStreamInfo>::new();
    for midi_device in midi_in_device_config.iter() {
        let port = client.register_port(&midi_device.device_name, jack::MidiIn::default())?;
        midi_in_ports.push(port);
        midi_in_stream_info.push(MidiDeviceStreamInfo {
            name: midi_device.device_name.clone(),
        });
    }

    let mut midi_out_ports = Vec::<jack::Port<jack::MidiOut>>::new();
    let mut midi_out_stream_info = Vec::<MidiDeviceStreamInfo>::new();
    for midi_device in midi_out_device_config.iter() {
        let port = client.register_port(&midi_device.device_name, jack::MidiOut::default())?;
        midi_out_ports.push(port);
        midi_out_stream_info.push(MidiDeviceStreamInfo {
            name: midi_device.device_name.clone(),
        });
    }

    let stream_info = StreamInfo {
        server_name: String::from("Jack"),
        audio_in_devices: audio_in_stream_info,
        audio_out_devices: audio_out_stream_info,
        midi_in_devices: midi_in_stream_info,
        midi_out_devices: midi_out_stream_info,
        sample_rate: sample_rate as u32,
        audio_buffer_size: BufferSizeInfo::MaximumSize(max_buffer_size),
    };

    rt_process_handler.init(&stream_info);

    let process = JackProcessHandler::new(
        rt_process_handler,
        audio_in_ports,
        audio_out_ports,
        &stream_info,
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

    let system_audio_in_ports: Vec<String> = async_client.as_client().ports(
        None,
        Some("32 bit float mono audio"),
        jack::PortFlags::IS_PHYSICAL | jack::PortFlags::IS_OUTPUT,
    );
    let system_audio_out_ports: Vec<String> = async_client.as_client().ports(
        None,
        Some("32 bit float mono audio"),
        jack::PortFlags::IS_PHYSICAL | jack::PortFlags::IS_INPUT,
    );

    let system_midi_in_ports: Vec<String> =
        async_client
            .as_client()
            .ports(None, Some("8 bit raw midi"), jack::PortFlags::IS_OUTPUT);
    let system_midi_out_ports: Vec<String> =
        async_client
            .as_client()
            .ports(None, Some("8 bit raw midi"), jack::PortFlags::IS_INPUT);

    // Ports will be in the correct order.
    for (in_port, wanted_in_port) in audio_in_port_names
        .iter()
        .zip(audio_in_port_connections.iter())
    {
        if system_audio_in_ports.contains(wanted_in_port) {
            async_client
                .as_client()
                .connect_ports_by_name(wanted_in_port, in_port)?;
        }
    }
    for (out_port, wanted_out_port) in audio_out_port_names
        .iter()
        .zip(audio_out_port_connections.iter())
    {
        if system_audio_out_ports.contains(wanted_out_port) {
            async_client
                .as_client()
                .connect_ports_by_name(out_port, wanted_out_port)?;
        }
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

    audio_in_buffers: Vec<AudioDeviceBuffer>,
    audio_out_buffers: Vec<AudioDeviceBuffer>,

    sample_rate: u32,
}

impl<P: RtProcessHandler> JackProcessHandler<P> {
    fn new(
        rt_process_handler: P,
        audio_in_ports: Vec<jack::Port<jack::AudioIn>>,
        audio_out_ports: Vec<jack::Port<jack::AudioOut>>,
        stream_info: &StreamInfo,
        max_buffer_size: u32,
    ) -> Self {
        let mut audio_in_buffers = Vec::<AudioDeviceBuffer>::new();
        for audio_device in stream_info.audio_in_devices.iter() {
            audio_in_buffers.push(AudioDeviceBuffer {
                device_name: audio_device.name.clone(),
                buffers: vec![Vec::with_capacity(max_buffer_size as usize)],
                frames: 0,
            });
        }

        let mut audio_out_buffers = Vec::<AudioDeviceBuffer>::new();
        for audio_device in stream_info.audio_out_devices.iter() {
            audio_out_buffers.push(AudioDeviceBuffer {
                device_name: audio_device.name.clone(),
                buffers: vec![Vec::with_capacity(max_buffer_size as usize)],
                frames: 0,
            });
        }

        Self {
            rt_process_handler,
            audio_in_ports,
            audio_out_ports,
            audio_in_buffers,
            audio_out_buffers,
            sample_rate: stream_info.sample_rate,
        }
    }
}

impl<P: RtProcessHandler> jack::ProcessHandler for JackProcessHandler<P> {
    fn process(&mut self, _: &jack::Client, ps: &jack::ProcessScope) -> jack::Control {
        let mut audio_frames = 0;

        // Ports should be in order. Jack devices only have one channel each.
        for (in_port, in_device) in self
            .audio_in_ports
            .iter()
            .zip(self.audio_in_buffers.iter_mut())
        {
            let in_slice = in_port.as_slice(ps);
            audio_frames = in_slice.len();

            // Jack devices only have one channel each.
            let buffer = &mut in_device.buffers[0];

            // This in theory will never actually allocate more memory because the vec
            // was preallocated with the maximum buffer size that jack will send.
            buffer.resize(in_slice.len(), 0.0);
            buffer.copy_from_slice(in_slice);
        }

        if self.audio_in_ports.len() == 0 {
            // No input channels, check output for audio_frames instead.
            if let Some(out_port) = self.audio_out_ports.first_mut() {
                audio_frames = out_port.as_mut_slice(ps).len();
            }
        }

        // Clear output buffers with zeros.
        for out_device in self.audio_out_buffers.iter_mut() {
            for channel in out_device.buffers_mut().iter_mut() {
                channel.clear();
                channel.resize(audio_frames, 0.0);
            }
        }

        self.rt_process_handler.process(ProcessInfo {
            audio_in: &self.audio_in_buffers,
            audio_out: &mut self.audio_out_buffers,
            audio_frames,

            sample_rate: self.sample_rate,
        });

        // Ports should be in order. Jack devices only have one channel each.

        for (out_port, out_device) in self
            .audio_out_ports
            .iter_mut()
            .zip(self.audio_out_buffers.iter())
        {
            let out_slice = out_port.as_mut_slice(ps);

            // Jack devices only have one channel each.
            let buffer = &out_device.buffers[0];

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
