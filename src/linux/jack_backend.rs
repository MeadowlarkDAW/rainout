use log::{debug, info, warn};

use crate::{
    AudioDeviceBuffer, AudioServerConfig, AudioServerInfo, BufferSizeInfo, DeviceIndex,
    DuplexDeviceInfo, InternalAudioDevice, InternalMidiDevice, MidiDeviceBuffer, MidiDeviceConfig,
    MidiServerInfo, ProcessInfo, RtProcessHandler, SpawnRtThreadError, StreamError, StreamInfo,
    SystemAudioDeviceInfo, SystemMidiDeviceInfo,
};

fn extract_device_name(port_name: &String) -> String {
    let mut i = 0;
    for c in port_name.chars() {
        if c == ':' {
            break;
        }
        i += 1;
    }

    String::from(&port_name[0..i])
}

pub fn refresh_audio_server(server: &mut AudioServerInfo) {
    info!("Refreshing list of available Jack audio devices...");

    server.devices.clear();

    match jack::Client::new("rustydaw_io_dummy_client", jack::ClientOptions::empty()) {
        Ok((client, _status)) => {
            let mut in_devices = Vec::<SystemAudioDeviceInfo>::new();
            let mut out_devices = Vec::<SystemAudioDeviceInfo>::new();

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

            for system_port_name in system_audio_in_ports.iter() {
                let system_device_name = extract_device_name(system_port_name);

                let mut push_device = true;
                for device in in_devices.iter_mut() {
                    if &device.name == &system_device_name {
                        device.channels += 1;
                        push_device = false;
                        break;
                    }
                }

                if push_device {
                    in_devices.push(SystemAudioDeviceInfo {
                        name: system_device_name.clone(),
                        channels: 1,
                    });

                    info!("Found Jack audio in device: {}", &system_device_name);
                }
            }

            for system_port_name in system_audio_out_ports.iter() {
                let system_device_name = extract_device_name(system_port_name);

                let mut push_device = true;
                for device in out_devices.iter_mut() {
                    if &device.name == &system_device_name {
                        device.channels += 1;
                        push_device = false;
                        break;
                    }
                }

                if push_device {
                    out_devices.push(SystemAudioDeviceInfo {
                        name: system_device_name.clone(),
                        channels: 1,
                    });

                    info!("Found Jack audio out device: {}", &system_device_name);
                }
            }

            server.devices.push(DuplexDeviceInfo {
                name: String::from("Jack"),
                in_devices,
                out_devices,
                sample_rates: vec![client.sample_rate() as u32],
                buffer_size: BufferSizeInfo::MaximumSize(client.buffer_size() as u32),
            });

            server.available = true;
        }
        Err(e) => {
            server.available = false;

            info!("Jack server is unavailable: {}", e);
        }
    }
}

pub fn refresh_midi_server(server: &mut MidiServerInfo) {
    info!("Refreshing list of available Jack MIDI devices...");

    server.in_devices.clear();
    server.out_devices.clear();

    match jack::Client::new("rustydaw_io_dummy_client", jack::ClientOptions::empty()) {
        Ok((client, _status)) => {
            let system_midi_in_ports: Vec<String> =
                client.ports(None, Some("8 bit raw midi"), jack::PortFlags::IS_OUTPUT);
            let system_midi_out_ports: Vec<String> =
                client.ports(None, Some("8 bit raw midi"), jack::PortFlags::IS_INPUT);

            for system_port_name in system_midi_in_ports.iter() {
                server.in_devices.push(SystemMidiDeviceInfo {
                    name: system_port_name.clone(),
                });

                info!("Found MIDI in port: {}", &system_port_name);
            }

            for system_port_name in system_midi_out_ports.iter() {
                server.out_devices.push(SystemMidiDeviceInfo {
                    name: system_port_name.clone(),
                });

                info!("Found MIDI out port: {}", &system_port_name);
            }

            server.available = true;
        }
        Err(e) => {
            server.available = false;

            info!("Jack server is unavailable: {}", e);
        }
    }
}

pub struct JackRtThreadHandle<P: RtProcessHandler, E>
where
    E: 'static + Send + Sync + FnOnce(StreamError),
{
    _async_client: jack::AsyncClient<JackNotificationHandler<E>, JackProcessHandler<P>>,
}

#[derive(Clone)]
struct JackSystemDevicePorts {
    name: String,
    ports: Vec<String>,
}

pub fn spawn_rt_thread<P: RtProcessHandler, E>(
    audio_config: &AudioServerConfig,
    create_midi_in_devices: &[MidiDeviceConfig],
    create_midi_out_devices: &[MidiDeviceConfig],
    mut rt_process_handler: P,
    error_callback: E,
    use_client_name: Option<String>,
) -> Result<(StreamInfo, JackRtThreadHandle<P, E>), SpawnRtThreadError>
where
    E: 'static + Send + Sync + FnOnce(StreamError),
{
    info!("Spawning Jack thread...");

    let client_name = use_client_name.unwrap_or(String::from("rusty-daw-io"));

    info!("Registering Jack client with name {}", &client_name);

    let (client, _status) = jack::Client::new(&client_name, jack::ClientOptions::empty())?;

    // Find system ports

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
    let mut system_in_devices = Vec::<JackSystemDevicePorts>::new();
    let mut system_out_devices = Vec::<JackSystemDevicePorts>::new();
    for system_port_name in system_audio_in_ports.iter() {
        let system_device_name = extract_device_name(system_port_name);

        let mut push_device = true;
        for device in system_in_devices.iter_mut() {
            if &device.name == &system_device_name {
                device.ports.push(system_port_name.clone());
                push_device = false;
                break;
            }
        }

        if push_device {
            system_in_devices.push(JackSystemDevicePorts {
                name: system_device_name.clone(),
                ports: vec![system_port_name.clone()],
            });

            info!("Found Jack audio in device: {}", &system_device_name);
        }
    }
    for system_port_name in system_audio_out_ports.iter() {
        let system_device_name = extract_device_name(system_port_name);

        let mut push_device = true;
        for device in system_out_devices.iter_mut() {
            if &device.name == &system_device_name {
                device.ports.push(system_port_name.clone());
                push_device = false;
                break;
            }
        }

        if push_device {
            system_out_devices.push(JackSystemDevicePorts {
                name: system_device_name.clone(),
                ports: vec![system_port_name.clone()],
            });

            info!("Found Jack audio out device: {}", &system_device_name);
        }
    }

    let system_in_device_name = audio_config.system_in_device.get_name_or("system");
    let system_out_device_name = audio_config.system_out_device.get_name_or("system");

    // Register new ports.

    let mut audio_in_ports = Vec::<jack::Port<jack::AudioIn>>::new();
    let mut audio_in_port_names = Vec::<String>::new();
    let mut audio_in_connected_port_names = Vec::<String>::new();
    let mut audio_in_devices = Vec::<InternalAudioDevice>::new();
    for (device_index, user_device) in audio_config.create_in_devices.iter().enumerate() {
        let mut system_device = None;
        for d in system_in_devices.iter() {
            if &d.name == system_in_device_name {
                system_device = Some(d);
                break;
            }
        }
        let system_device = system_device.ok_or_else(|| {
            SpawnRtThreadError::SystemAudioInDeviceNotFound(String::from(system_in_device_name))
        })?;

        let use_channels = user_device
            .system_channels
            .as_channel_index_array(system_device.ports.len() as u16);

        if use_channels.len() == 0 {
            return Err(SpawnRtThreadError::NoSystemChannelsGiven(
                user_device.id.clone(),
            ));
        }

        audio_in_devices.push(InternalAudioDevice {
            id_name: user_device.id.clone(),
            id_index: DeviceIndex::new(device_index),
            system_device: String::from(system_in_device_name),
            system_channels: use_channels.clone(),
            channels: use_channels.len() as u16,
        });

        for (i, system_channel) in use_channels.iter().enumerate() {
            let system_port_name = match system_device.ports.get(usize::from(*system_channel)) {
                Some(n) => n,
                None => {
                    return Err(SpawnRtThreadError::SystemInChannelNotFound(
                        system_device.name.clone(),
                        *system_channel,
                    ));
                }
            };

            let port_name = format!("{}_{}", &user_device.id, i + 1);
            let port = client.register_port(&port_name, jack::AudioIn::default())?;

            audio_in_port_names.push(port.name()?);
            audio_in_connected_port_names.push(system_port_name.clone());
            audio_in_ports.push(port);
        }
    }

    let mut audio_out_ports = Vec::<jack::Port<jack::AudioOut>>::new();
    let mut audio_out_port_names = Vec::<String>::new();
    let mut audio_out_connected_port_names = Vec::<String>::new();
    let mut audio_out_devices = Vec::<InternalAudioDevice>::new();
    for (device_index, user_device) in audio_config.create_out_devices.iter().enumerate() {
        let mut system_device = None;
        for d in system_out_devices.iter() {
            if &d.name == system_out_device_name {
                system_device = Some(d);
                break;
            }
        }
        let system_device = system_device.ok_or_else(|| {
            SpawnRtThreadError::SystemAudioOutDeviceNotFound(String::from(system_out_device_name))
        })?;

        let use_channels = user_device
            .system_channels
            .as_channel_index_array(system_device.ports.len() as u16);

        if use_channels.len() == 0 {
            return Err(SpawnRtThreadError::NoSystemChannelsGiven(
                user_device.id.clone(),
            ));
        }

        audio_out_devices.push(InternalAudioDevice {
            id_name: user_device.id.clone(),
            id_index: DeviceIndex::new(device_index),
            system_device: String::from(system_out_device_name),
            system_channels: use_channels.clone(),
            channels: use_channels.len() as u16,
        });

        for (i, system_channel) in use_channels.iter().enumerate() {
            let system_port_name = match system_device.ports.get(usize::from(*system_channel)) {
                Some(n) => n,
                None => {
                    return Err(SpawnRtThreadError::SystemInChannelNotFound(
                        system_device.name.clone(),
                        *system_channel,
                    ));
                }
            };

            let port_name = format!("{}_{}", &user_device.id, i + 1);
            let port = client.register_port(&port_name, jack::AudioOut::default())?;

            audio_out_port_names.push(port.name()?);
            audio_out_connected_port_names.push(system_port_name.clone());
            audio_out_ports.push(port);
        }
    }

    let mut midi_in_ports = Vec::<jack::Port<jack::MidiIn>>::new();
    let mut midi_in_port_names = Vec::<String>::new();
    let mut midi_in_connected_port_names = Vec::<String>::new();
    let mut midi_in_devices = Vec::<InternalMidiDevice>::new();
    for (device_index, midi_device) in create_midi_in_devices.iter().enumerate() {
        let system_port_name = &midi_device.system_port;

        midi_in_devices.push(InternalMidiDevice {
            id_name: midi_device.id.clone(),
            id_index: DeviceIndex::new(device_index),
            system_port: String::from(system_port_name),
        });

        let port = client.register_port(&midi_device.id, jack::MidiIn::default())?;

        midi_in_port_names.push(port.name()?);
        midi_in_connected_port_names.push(String::from(system_port_name));
        midi_in_ports.push(port);
    }

    let mut midi_out_ports = Vec::<jack::Port<jack::MidiOut>>::new();
    let mut midi_out_port_names = Vec::<String>::new();
    let mut midi_out_connected_port_names = Vec::<String>::new();
    let mut midi_out_devices = Vec::<InternalMidiDevice>::new();
    for (device_index, midi_device) in create_midi_out_devices.iter().enumerate() {
        let system_port_name = &midi_device.system_port;

        midi_out_devices.push(InternalMidiDevice {
            id_name: midi_device.id.clone(),
            id_index: DeviceIndex::new(device_index),
            system_port: String::from(system_port_name),
        });

        let port = client.register_port(&midi_device.id, jack::MidiOut::default())?;

        midi_out_port_names.push(port.name()?);
        midi_out_connected_port_names.push(String::from(system_port_name));
        midi_out_ports.push(port);
    }

    let sample_rate = client.sample_rate() as u32;
    let max_audio_buffer_size = client.buffer_size() as u32;

    let stream_info = StreamInfo {
        server_name: String::from("Jack"),
        audio_in: audio_in_devices,
        audio_out: audio_out_devices,
        midi_in: midi_in_devices,
        midi_out: midi_out_devices,
        sample_rate: sample_rate as u32,
        audio_buffer_size: BufferSizeInfo::MaximumSize(max_audio_buffer_size),
    };

    rt_process_handler.init(&stream_info);

    let process = JackProcessHandler::new(
        rt_process_handler,
        audio_in_ports,
        audio_out_ports,
        midi_in_ports,
        midi_out_ports,
        stream_info.clone(),
        max_audio_buffer_size,
    );

    info!("Activating Jack client...");

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
        async_client
            .as_client()
            .connect_ports_by_name(&system_in_port, in_port)?;
    }
    for (out_port, system_out_port) in audio_out_port_names
        .iter()
        .zip(audio_out_connected_port_names)
    {
        async_client
            .as_client()
            .connect_ports_by_name(out_port, &system_out_port)?;
    }

    for (in_port, system_in_port) in midi_in_port_names.iter().zip(midi_in_connected_port_names) {
        async_client
            .as_client()
            .connect_ports_by_name(&system_in_port, in_port)?;
    }
    for (out_port, system_out_port) in midi_out_port_names
        .iter()
        .zip(midi_out_connected_port_names)
    {
        async_client
            .as_client()
            .connect_ports_by_name(out_port, &system_out_port)?;
    }

    info!(
        "Successfully spawned Jack thread. Sample rate: {}, Max audio buffer size: {}",
        sample_rate, max_audio_buffer_size
    );

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

    midi_in_ports: Vec<jack::Port<jack::MidiIn>>,
    midi_out_ports: Vec<jack::Port<jack::MidiOut>>,

    midi_in_buffers: Vec<MidiDeviceBuffer>,
    midi_out_buffers: Vec<MidiDeviceBuffer>,

    stream_info: StreamInfo,
    max_audio_buffer_size: usize,
}

impl<P: RtProcessHandler> JackProcessHandler<P> {
    fn new(
        rt_process_handler: P,
        audio_in_ports: Vec<jack::Port<jack::AudioIn>>,
        audio_out_ports: Vec<jack::Port<jack::AudioOut>>,
        midi_in_ports: Vec<jack::Port<jack::MidiIn>>,
        midi_out_ports: Vec<jack::Port<jack::MidiOut>>,
        stream_info: StreamInfo,
        max_audio_buffer_size: u32,
    ) -> Self {
        let mut audio_in_buffers = Vec::<AudioDeviceBuffer>::new();
        let mut audio_out_buffers = Vec::<AudioDeviceBuffer>::new();

        for device in stream_info.audio_in.iter() {
            audio_in_buffers.push(AudioDeviceBuffer::new(
                device.channels,
                max_audio_buffer_size,
            ))
        }
        for device in stream_info.audio_out.iter() {
            audio_out_buffers.push(AudioDeviceBuffer::new(
                device.channels,
                max_audio_buffer_size,
            ))
        }

        let mut midi_in_buffers = Vec::<MidiDeviceBuffer>::new();
        let mut midi_out_buffers = Vec::<MidiDeviceBuffer>::new();

        for _ in 0..stream_info.midi_in.len() {
            midi_in_buffers.push(MidiDeviceBuffer::new())
        }
        for _ in 0..stream_info.midi_out.len() {
            midi_out_buffers.push(MidiDeviceBuffer::new())
        }

        Self {
            rt_process_handler,
            audio_in_ports,
            audio_out_ports,
            audio_in_buffers,
            audio_out_buffers,
            midi_in_ports,
            midi_out_ports,
            midi_in_buffers,
            midi_out_buffers,
            stream_info,
            max_audio_buffer_size: max_audio_buffer_size as usize,
        }
    }
}

impl<P: RtProcessHandler> jack::ProcessHandler for JackProcessHandler<P> {
    fn process(&mut self, _: &jack::Client, ps: &jack::ProcessScope) -> jack::Control {
        let mut audio_frames = 0;

        // Collect Audio Inputs

        let mut port = 0; // Ports are in order.
        for audio_buffer in self.audio_in_buffers.iter_mut() {
            for channel in audio_buffer.channel_buffers.iter_mut() {
                let port_slice = self.audio_in_ports[port].as_slice(ps);

                audio_frames = port_slice.len();

                // Sanity check.
                if audio_frames > self.max_audio_buffer_size {
                    warn!("Warning: Jack sent a buffer size of {} when the max buffer size was said to be {}", audio_frames, self.max_audio_buffer_size);
                }

                // The compiler should in-theory optimize by not filling in zeros before copying
                // the slice. This should never allocate because each buffer was given a capacity of
                // the maximum buffer size that jack will send.
                channel.resize(audio_frames, 0.0);
                channel.copy_from_slice(port_slice);

                port += 1;
            }

            audio_buffer.frames = audio_frames;
        }

        if self.audio_in_buffers.len() == 0 {
            // Check outputs for number of frames instead.
            if let Some(out_port) = self.audio_out_ports.first_mut() {
                audio_frames = out_port.as_mut_slice(ps).len();
            }
        }

        // Clear Audio Outputs

        for audio_buffer in self.audio_out_buffers.iter_mut() {
            audio_buffer.clear_and_resize(audio_frames);
        }

        // Collect MIDI Inputs

        for (midi_buffer, port) in self
            .midi_in_buffers
            .iter_mut()
            .zip(self.midi_in_ports.iter())
        {
            midi_buffer.clear();

            for event in port.iter(ps) {
                if let Err(e) = midi_buffer.push_raw(event.time, event.bytes) {
                    warn!(
                        "Warning: Dropping midi event because of the push error: {}",
                        e
                    );
                }
            }
        }

        // Clear MIDI Outputs

        for midi_buffer in self.midi_out_buffers.iter_mut() {
            midi_buffer.clear();
        }

        self.rt_process_handler.process(ProcessInfo {
            audio_in: self.audio_in_buffers.as_slice(),
            audio_out: self.audio_out_buffers.as_mut_slice(),
            audio_frames,

            midi_in: self.midi_in_buffers.as_slice(),
            midi_out: self.midi_out_buffers.as_mut_slice(),

            sample_rate: self.stream_info.sample_rate,
        });

        // Copy processed data to Audio Outputs

        let mut port = 0; // Ports are in order.
        for audio_buffer in self.audio_out_buffers.iter() {
            for channel in audio_buffer.channel_buffers.iter() {
                let port_slice = self.audio_out_ports[port].as_mut_slice(ps);

                // Just in case the user resized the output buffer for some reason.
                let len = channel.len().min(port_slice.len());
                if len != audio_frames {
                    warn!(
                        "Warning: An audio output buffer was resized from {} to {} by the user",
                        audio_frames, len
                    );
                }

                &mut port_slice[0..len].copy_from_slice(&channel[0..len]);

                port += 1;
            }
        }

        // Copy processed data to MIDI Outputs

        for (midi_buffer, port) in self
            .midi_out_buffers
            .iter()
            .zip(self.midi_out_ports.iter_mut())
        {
            let mut port_writer = port.writer(ps);

            for event in midi_buffer.events() {
                if let Err(e) = port_writer.write(&jack::RawMidi {
                    time: event.delta_frames,
                    bytes: &event.data(),
                }) {
                    warn!("Warning: Could not copy midi data to Jack output: {}", e);
                }
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
        debug!("JACK: thread init");
    }

    fn shutdown(&mut self, status: jack::ClientStatus, reason: &str) {
        let msg = format!(
            "JACK: shutdown with status {:?} because \"{}\"",
            status, reason
        );

        info!("{}", msg);

        if let Some(error_callback) = self.error_callback.take() {
            (error_callback)(StreamError::AudioServerDisconnected(msg))
        }
    }

    fn freewheel(&mut self, _: &jack::Client, is_enabled: bool) {
        debug!(
            "JACK: freewheel mode is {}",
            if is_enabled { "on" } else { "off" }
        );
    }

    fn sample_rate(&mut self, _: &jack::Client, srate: jack::Frames) -> jack::Control {
        debug!("JACK: sample rate changed to {}", srate);
        jack::Control::Continue
    }

    fn client_registration(&mut self, _: &jack::Client, name: &str, is_reg: bool) {
        debug!(
            "JACK: {} client with name \"{}\"",
            if is_reg { "registered" } else { "unregistered" },
            name
        );
    }

    fn port_registration(&mut self, _: &jack::Client, port_id: jack::PortId, is_reg: bool) {
        debug!(
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
        debug!(
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
        debug!(
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
        debug!("JACK: graph reordered");
        jack::Control::Continue
    }

    fn xrun(&mut self, _: &jack::Client) -> jack::Control {
        warn!("JACK: xrun occurred");
        jack::Control::Continue
    }

    fn latency(&mut self, _: &jack::Client, mode: jack::LatencyType) {
        debug!(
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
