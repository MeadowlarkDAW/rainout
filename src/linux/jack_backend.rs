use std::collections::HashSet;

use crate::{
    AudioDeviceBuffer, AudioDeviceConfig, AudioServerInfo, BufferSizeInfo, ConnectionType,
    DeviceIndex, InternalAudioDevice, InternalMidiDevice, MidiDeviceConfig, MidiServerInfo,
    ProcessInfo, RtProcessHandler, SpawnRtThreadError, StreamError, StreamInfo,
    SystemAudioDeviceInfo, SystemMidiDeviceInfo,
};

pub fn refresh_audio_server(server: &mut AudioServerInfo) {
    server.system_in_devices.clear();
    server.system_out_devices.clear();

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
                Some("system"),
                Some("32 bit float mono audio"),
                jack::PortFlags::IS_OUTPUT,
            );
            let system_audio_out_ports: Vec<String> = client.ports(
                Some("system"),
                Some("32 bit float mono audio"),
                jack::PortFlags::IS_INPUT,
            );

            if system_audio_in_ports.len() > 0 {
                server.system_in_devices.push(SystemAudioDeviceInfo {
                    name: String::from("system"),
                    ports: system_audio_in_ports,
                });
            }
            if system_audio_out_ports.len() > 0 {
                server.system_out_devices.push(SystemAudioDeviceInfo {
                    name: String::from("system"),
                    ports: system_audio_out_ports,
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
    /*
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
    */
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

    // Register new ports.

    let mut audio_in_ports = Vec::<jack::Port<jack::AudioIn>>::new();
    let mut audio_in_port_names = Vec::<String>::new();
    let mut audio_in_connected_port_names = Vec::<Option<String>>::new();
    let mut audio_in_devices = Vec::<InternalAudioDevice>::new();
    for (i, audio_device) in audio_in_device_config.iter().enumerate() {
        // See if device wants to be connected to a system device.
        match &audio_device.connection {
            ConnectionType::SystemPorts { ports } => {
                if ports.len() == 0 {
                    return Err(SpawnRtThreadError::NoSystemPortsGiven(
                        audio_device.id.clone(),
                    ));
                }

                audio_in_devices.push(InternalAudioDevice {
                    id_name: audio_device.id.clone(),
                    id_index: DeviceIndex::new(i),
                    connection: audio_device.connection.clone(),
                    channels: ports.len() as u16,
                });

                for (i, system_port_name) in ports.iter().enumerate() {
                    let port_name = format!("{}_{}", &audio_device.id, i + 1);
                    let port = client.register_port(&port_name, jack::AudioIn::default())?;

                    audio_in_port_names.push(port.name()?);
                    audio_in_connected_port_names.push(Some(system_port_name.clone()));
                    audio_in_ports.push(port);
                }
            }
            ConnectionType::Virtual { channels } => {
                if *channels == 0 {
                    return Err(SpawnRtThreadError::EmptyVirtualDevice(
                        audio_device.id.clone(),
                    ));
                }

                audio_in_devices.push(InternalAudioDevice {
                    id_name: audio_device.id.clone(),
                    id_index: DeviceIndex::new(i),
                    connection: audio_device.connection.clone(),
                    channels: *channels,
                });

                for i in 0..*channels {
                    let port_name = format!("{}_{}", &audio_device.id, i + 1);
                    let port = client.register_port(&port_name, jack::AudioIn::default())?;

                    audio_in_port_names.push(port.name()?);
                    audio_in_connected_port_names.push(None);
                    audio_in_ports.push(port);
                }
            }
        }
    }

    let mut audio_out_ports = Vec::<jack::Port<jack::AudioOut>>::new();
    let mut audio_out_port_names = Vec::<String>::new();
    let mut audio_out_connected_port_names = Vec::<Option<String>>::new();
    let mut audio_out_devices = Vec::<InternalAudioDevice>::new();
    for (i, audio_device) in audio_out_device_config.iter().enumerate() {
        // See if device wants to be connected to a system device.
        match &audio_device.connection {
            ConnectionType::SystemPorts { ports } => {
                if ports.len() == 0 {
                    return Err(SpawnRtThreadError::NoSystemPortsGiven(
                        audio_device.id.clone(),
                    ));
                }

                audio_out_devices.push(InternalAudioDevice {
                    id_name: audio_device.id.clone(),
                    id_index: DeviceIndex::new(i),
                    connection: audio_device.connection.clone(),
                    channels: ports.len() as u16,
                });

                for (i, system_port_name) in ports.iter().enumerate() {
                    let port_name = format!("{}_{}", &audio_device.id, i + 1);
                    let port = client.register_port(&port_name, jack::AudioOut::default())?;

                    audio_out_port_names.push(port.name()?);
                    audio_out_connected_port_names.push(Some(system_port_name.clone()));
                    audio_out_ports.push(port);
                }
            }
            ConnectionType::Virtual { channels } => {
                if *channels == 0 {
                    return Err(SpawnRtThreadError::EmptyVirtualDevice(
                        audio_device.id.clone(),
                    ));
                }

                audio_out_devices.push(InternalAudioDevice {
                    id_name: audio_device.id.clone(),
                    id_index: DeviceIndex::new(i),
                    connection: audio_device.connection.clone(),
                    channels: *channels,
                });

                for i in 0..*channels {
                    let port_name = format!("{}_{}", &audio_device.id, i + 1);
                    let port = client.register_port(&port_name, jack::AudioOut::default())?;

                    audio_out_port_names.push(port.name()?);
                    audio_out_connected_port_names.push(None);
                    audio_out_ports.push(port);
                }
            }
        }
    }

    let sample_rate = client.sample_rate() as u32;
    let max_audio_buffer_size = client.buffer_size() as u32;

    let stream_info = StreamInfo {
        server_name: String::from("Jack"),
        audio_in: audio_in_devices,
        audio_out: audio_out_devices,
        midi_in: vec![],
        midi_out: vec![],
        sample_rate: sample_rate as u32,
        audio_buffer_size: BufferSizeInfo::MaximumSize(max_audio_buffer_size),
    };

    rt_process_handler.init(&stream_info);

    let process = JackProcessHandler::new(
        rt_process_handler,
        audio_in_ports,
        audio_out_ports,
        stream_info.clone(),
        max_audio_buffer_size as usize,
    );

    // Activate the client, which starts the processing.
    let async_client = client.activate_async(
        JackNotificationHandler {
            error_callback: Some(error_callback),
        },
        process,
    )?;

    // Try to automatically connect to system inputs/outputs.

    for (in_port, system_in_port) in audio_in_port_names
        .iter()
        .zip(audio_in_connected_port_names)
    {
        if let Some(system_in_port) = &system_in_port {
            async_client
                .as_client()
                .connect_ports_by_name(system_in_port, in_port)?;
        }
    }
    for (out_port, system_out_port) in audio_out_port_names
        .iter()
        .zip(audio_out_connected_port_names)
    {
        if let Some(system_out_port) = &system_out_port {
            async_client
                .as_client()
                .connect_ports_by_name(out_port, system_out_port)?;
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

    stream_info: StreamInfo,
    max_audio_buffer_size: usize,
}

impl<P: RtProcessHandler> JackProcessHandler<P> {
    fn new(
        rt_process_handler: P,
        audio_in_ports: Vec<jack::Port<jack::AudioIn>>,
        audio_out_ports: Vec<jack::Port<jack::AudioOut>>,
        stream_info: StreamInfo,
        max_audio_buffer_size: usize,
    ) -> Self {
        let mut audio_in_buffers = Vec::<AudioDeviceBuffer>::new();
        let mut audio_out_buffers = Vec::<AudioDeviceBuffer>::new();

        for internal_device in stream_info.audio_in.iter() {
            let mut channels = Vec::<Vec<f32>>::new();
            for _ in 0..internal_device.channels {
                channels.push(Vec::<f32>::with_capacity(max_audio_buffer_size));
            }

            audio_in_buffers.push(AudioDeviceBuffer {
                channels,
                frames: 0,
            });
        }

        for internal_device in stream_info.audio_out.iter() {
            let mut channels = Vec::<Vec<f32>>::new();
            for _ in 0..internal_device.channels {
                channels.push(Vec::<f32>::with_capacity(max_audio_buffer_size));
            }

            audio_out_buffers.push(AudioDeviceBuffer {
                channels,
                frames: 0,
            });
        }

        Self {
            rt_process_handler,
            audio_in_ports,
            audio_out_ports,
            audio_in_buffers,
            audio_out_buffers,
            stream_info,
            max_audio_buffer_size,
        }
    }
}

impl<P: RtProcessHandler> jack::ProcessHandler for JackProcessHandler<P> {
    fn process(&mut self, _: &jack::Client, ps: &jack::ProcessScope) -> jack::Control {
        let mut audio_frames = 0;

        // Copy all inputs into buffers.
        let mut in_port = 0; // Ports are in order.
        for in_device_buffer in self.audio_in_buffers.iter_mut() {
            for channel in in_device_buffer.channels.iter_mut() {
                let in_port_slice = self.audio_in_ports[in_port].as_slice(ps);

                audio_frames = in_port_slice.len();

                // Sanity check.
                if audio_frames > self.max_audio_buffer_size {
                    println!("Warning: Jack sent a buffer size of {} when the max buffer size was said to be {}", audio_frames, self.max_audio_buffer_size);
                }

                // The compiler should in-theory optimize by not filling in zeros before copying
                // the slice. This should never allocate because each buffer was given a capacity of
                // the maximum buffer size that jack will send.
                channel.resize(audio_frames, 0.0);
                channel.copy_from_slice(in_port_slice);

                in_port += 1;
            }

            in_device_buffer.frames = audio_frames;
        }

        if self.audio_in_buffers.len() == 0 {
            // Check outputs for number of frames instead.
            if let Some(out_port) = self.audio_out_ports.first_mut() {
                audio_frames = out_port.as_mut_slice(ps).len();
            }
        }

        // Clear all output buffers to zeros.
        for out_device_buffer in self.audio_out_buffers.iter_mut() {
            out_device_buffer.clear_and_resize(audio_frames);
        }

        self.rt_process_handler.process(ProcessInfo {
            audio_in: self.audio_in_buffers.as_slice(),
            audio_out: self.audio_out_buffers.as_mut_slice(),
            audio_frames,

            sample_rate: self.stream_info.sample_rate,
        });

        // Copy new data to outputs.
        let mut out_port = 0; // Ports are in order.
        for out_device_buffer in self.audio_out_buffers.iter() {
            for channel in out_device_buffer.channels.iter() {
                let out_port_slice = self.audio_out_ports[out_port].as_mut_slice(ps);

                // Just in case the user resized the output buffer for some reason.
                let len = channel.len().min(out_port_slice.len());
                if len != audio_frames {
                    println!(
                        "Warning: An audio output buffer was resized from {} to {} by the user",
                        audio_frames, len
                    );
                }

                &mut out_port_slice[0..len].copy_from_slice(&channel[0..len]);

                out_port += 1;
            }
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
