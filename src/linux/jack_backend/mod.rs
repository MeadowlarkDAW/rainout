use crate::error::{
    ChangeAudioBufferSizeError, ChangeAudioPortConfigError, FatalStreamError, RunConfigError,
};
use crate::error_behavior::AudioPortNotFoundBehavior;
use crate::{
    AudioBackend, AudioBackendInfo, AudioBufferSizeConfig, AudioDeviceInfo, Config,
    DefaultChannelLayout, DeviceID, ErrorHandler, FixedBufferSizeRange, PlatformStreamHandle,
    ProcessHandler, ProcessInfo, RunOptions, StreamAudioBufferSize, StreamAudioPortInfo,
    StreamHandle, StreamInfo,
};

#[cfg(feature = "midi")]
use crate::{
    error::{ChangeMidiDeviceConfigError, MidiBufferPushError},
    error_behavior::MidiDeviceNotFoundBehavior,
    MidiBackend, MidiBackendInfo, MidiBuffer, MidiDeviceInfo, MidiStreamInfo, StreamMidiDeviceInfo,
};

const DUMMY_CLIENT_NAME: &'static str = "rustydaw_io_dummy_client";
const DEFAULT_CLIENT_NAME: &'static str = "rustydaw_io_client";
const JACK_DEVICE_NAME: &'static str = "Jack Server Device";

pub fn enumerate_audio_backend() -> AudioBackendInfo {
    log::debug!("Enumerating Jack server...");

    match jack::Client::new(DUMMY_CLIENT_NAME, jack::ClientOptions::empty()) {
        Ok((client, _status)) => {
            let system_audio_in_ports: Vec<String> =
                client.ports(None, Some("32 bit float mono audio"), jack::PortFlags::IS_OUTPUT);
            let system_audio_out_ports: Vec<String> =
                client.ports(None, Some("32 bit float mono audio"), jack::PortFlags::IS_INPUT);

            // Find index of default input port.
            let mut default_in_port = 0; // Fallback to first available port.
            for (i, port) in system_audio_in_ports.iter().enumerate() {
                if port == "system:capture_1" {
                    default_in_port = i;
                    break;
                }
            }
            let default_input_layout = if !system_audio_in_ports.is_empty() {
                DefaultChannelLayout::Mono(default_in_port)
            } else {
                DefaultChannelLayout::Unspecified
            };

            // Find index of default out left port.
            let mut default_out_port_left = 0; // Fallback to first available port.
            for (i, port) in system_audio_out_ports.iter().enumerate() {
                if port == "system:playback_1" {
                    default_out_port_left = i;
                    break;
                }
            }
            // Find index of default out right port.
            let mut default_out_port_right = 1.min(system_audio_out_ports.len() - 1); // Fallback to second available port if stereo, first if mono.
            for (i, port) in system_audio_out_ports.iter().enumerate() {
                if port == "system:playback_2" {
                    default_out_port_right = i;
                    break;
                }
            }
            let default_output_layout = if !system_audio_out_ports.is_empty() {
                if system_audio_in_ports.len() == 1
                    || default_out_port_left == default_out_port_right
                {
                    DefaultChannelLayout::Mono(default_out_port_left)
                } else {
                    DefaultChannelLayout::Stereo {
                        left: default_out_port_left,
                        right: default_out_port_right,
                    }
                }
            } else {
                DefaultChannelLayout::Unspecified
            };

            // Only one sample rate is available which is the sample rate configured
            // for the server.
            let sample_rate = client.sample_rate() as u32;

            // Only one fixed buffer size is available which is the buffer size
            // configured for the server.
            let buffer_size = client.buffer_size() as u32;

            // Jack only ever has one "device" which is the audio server itself.
            let device = AudioDeviceInfo {
                id: DeviceID { name: String::from(JACK_DEVICE_NAME), unique_id: None },
                in_ports: system_audio_in_ports,
                out_ports: system_audio_out_ports,
                sample_rates: vec![sample_rate],
                default_sample_rate: sample_rate,
                fixed_buffer_size_range: Some(FixedBufferSizeRange {
                    min: buffer_size,
                    max: buffer_size,
                    must_be_power_of_2: true,
                    default: buffer_size,
                }),
                default_input_layout,
                default_output_layout,
            };

            return AudioBackendInfo {
                backend: AudioBackend::JackLinux,
                version: None,
                running: true,
                devices: vec![device],
                default_device: Some(0),
            };
        }
        Err(e) => {
            log::warn!("Jack server is unavailable: {}", e);
        }
    }

    AudioBackendInfo {
        backend: AudioBackend::JackLinux,
        version: None,
        running: false,
        devices: Vec::new(),
        default_device: None,
    }
}

#[cfg(feature = "midi")]
pub fn enumerate_midi_backend() -> MidiBackendInfo {
    log::debug!("Enumerating Jack MIDI server...");

    match jack::Client::new(DUMMY_CLIENT_NAME, jack::ClientOptions::empty()) {
        Ok((client, _status)) => {
            let in_devices: Vec<MidiDeviceInfo> = client
                .ports(None, Some("8 bit raw midi"), jack::PortFlags::IS_OUTPUT)
                .drain(..)
                .map(|n| MidiDeviceInfo { id: DeviceID { name: n, unique_id: None } })
                .collect();
            let out_devices: Vec<MidiDeviceInfo> = client
                .ports(None, Some("8 bit raw midi"), jack::PortFlags::IS_INPUT)
                .drain(..)
                .map(|n| MidiDeviceInfo { id: DeviceID { name: n, unique_id: None } })
                .collect();

            // Find index of the default in port.
            let mut default_in_port = 0; // Fallback to first available port.
            for (i, device) in in_devices.iter().enumerate() {
                // "system:midi_capture_1" is usually Jack's built-in `Midi-Through` device.
                // What we usually want is first available port of the user's hardware MIDI controller, which is
                // commonly mapped to "system:midi_capture_2".
                if &device.id.name == "system:midi_capture_2" {
                    default_in_port = i;
                    break;
                }
            }
            let default_in_device =
                if in_devices.is_empty() { None } else { Some(default_in_port) };

            // Find index of the default out port.
            let mut default_out_port = 0; // Fallback to first available port.
            for (i, device) in out_devices.iter().enumerate() {
                // "system:midi_capture_1" is usually Jack's built-in `Midi-Through` device.
                // What we usually want is first available port of the user's hardware MIDI controller, which is
                // commonly mapped to "system:midi_playback_2".
                if &device.id.name == "system:midi_playback_2" {
                    default_out_port = i;
                    break;
                }
            }
            let default_out_device =
                if out_devices.is_empty() { None } else { Some(default_out_port) };

            return MidiBackendInfo {
                backend: MidiBackend::JackLinux,
                version: None,
                running: true,
                in_devices,
                out_devices,
                default_in_device,
                default_out_device,
            };
        }
        Err(e) => {
            log::warn!("Jack server is unavailable: {}", e);
        }
    }

    MidiBackendInfo {
        backend: MidiBackend::JackLinux,
        version: None,
        running: false,
        in_devices: Vec::new(),
        out_devices: Vec::new(),
        default_in_device: None,
        default_out_device: None,
    }
}

pub fn run<P: ProcessHandler, E: ErrorHandler>(
    config: &Config,
    options: &RunOptions,
    mut process_handler: P,
    error_handler: E,
) -> Result<StreamHandle<P, E>, RunConfigError> {
    log::debug!("Spawning Jack thread...");

    let client_name =
        options.use_application_name.clone().unwrap_or(String::from(DEFAULT_CLIENT_NAME));

    log::debug!("Registering Jack client with name {}", &client_name);

    let (client, _status) = jack::Client::new(&client_name, jack::ClientOptions::empty())?;

    // Find system ports
    let system_audio_in_ports: Vec<String> =
        client.ports(None, Some("32 bit float mono audio"), jack::PortFlags::IS_OUTPUT);
    let system_audio_out_ports: Vec<String> =
        client.ports(None, Some("32 bit float mono audio"), jack::PortFlags::IS_INPUT);

    // Register new ports.
    let mut client_audio_in_ports =
        Vec::<jack::Port<jack::AudioIn>>::with_capacity(config.audio_in_ports.len());
    let mut client_audio_in_port_names = Vec::<String>::with_capacity(config.audio_in_ports.len());
    let mut client_audio_in_connected_to =
        Vec::<Option<String>>::with_capacity(config.audio_in_ports.len());
    let mut audio_in_port_info =
        Vec::<StreamAudioPortInfo>::with_capacity(config.audio_in_ports.len());

    let mut client_audio_out_ports =
        Vec::<jack::Port<jack::AudioOut>>::with_capacity(config.audio_out_ports.len());
    let mut client_audio_out_port_names =
        Vec::<String>::with_capacity(config.audio_out_ports.len());
    let mut client_audio_out_connected_to =
        Vec::<Option<String>>::with_capacity(config.audio_out_ports.len());
    let mut audio_out_port_info =
        Vec::<StreamAudioPortInfo>::with_capacity(config.audio_out_ports.len());

    for (i, port) in config.audio_in_ports.iter().enumerate() {
        if !system_audio_in_ports.contains(port) {
            if let AudioPortNotFoundBehavior::ReturnWithError =
                options.error_behavior.audio_port_not_found
            {
                return Err(RunConfigError::AudioPortNotFound(port.clone()));
            }
            client_audio_in_connected_to.push(None);
            audio_in_port_info.push(StreamAudioPortInfo { name: port.clone(), success: false });
        } else {
            client_audio_in_connected_to.push(Some(port.clone()));
            audio_in_port_info.push(StreamAudioPortInfo { name: port.clone(), success: true });
        }

        let client_port_name = format!("in_{}", i + 1);
        let client_port = client.register_port(&client_port_name, jack::AudioIn::default())?;

        client_audio_in_ports.push(client_port);
        client_audio_in_port_names.push(client_port_name);
    }

    for (i, port) in config.audio_out_ports.iter().enumerate() {
        if !system_audio_out_ports.contains(port) {
            if let AudioPortNotFoundBehavior::ReturnWithError =
                options.error_behavior.audio_port_not_found
            {
                return Err(RunConfigError::AudioPortNotFound(port.clone()));
            }
            client_audio_out_connected_to.push(None);
            audio_out_port_info.push(StreamAudioPortInfo { name: port.clone(), success: false });
        } else {
            client_audio_out_connected_to.push(Some(port.clone()));
            audio_out_port_info.push(StreamAudioPortInfo { name: port.clone(), success: true });
        }

        let client_port_name = format!("out_{}", i + 1);
        let client_port = client.register_port(&client_port_name, jack::AudioOut::default())?;

        client_audio_out_ports.push(client_port);
        client_audio_out_port_names.push(client_port_name);
    }

    #[cfg(feature = "midi")]
    struct MidiPortInfo {
        client_midi_in_port_names: Vec<String>,
        client_midi_in_connected_to: Vec<Option<String>>,
        midi_in_port_info: Vec<StreamMidiDeviceInfo>,

        client_midi_out_port_names: Vec<String>,
        client_midi_out_connected_to: Vec<Option<String>>,
        midi_out_port_info: Vec<StreamMidiDeviceInfo>,
    }

    #[cfg(feature = "midi")]
    let (client_midi_in_ports, client_midi_out_ports, midi_port_info) = {
        if let Some(midi_config) = &config.midi_config {
            if let MidiBackend::JackLinux = midi_config.backend {
                let system_midi_in_ports: Vec<String> =
                    client.ports(None, Some("8 bit raw midi"), jack::PortFlags::IS_OUTPUT);
                let system_midi_out_ports: Vec<String> =
                    client.ports(None, Some("8 bit raw midi"), jack::PortFlags::IS_INPUT);

                let mut client_midi_in_ports =
                    Vec::<jack::Port<jack::MidiIn>>::with_capacity(midi_config.in_devices.len());
                let mut client_midi_in_port_names =
                    Vec::<String>::with_capacity(midi_config.in_devices.len());
                let mut client_midi_in_connected_to =
                    Vec::<Option<String>>::with_capacity(midi_config.in_devices.len());
                let mut midi_in_port_info =
                    Vec::<StreamMidiDeviceInfo>::with_capacity(midi_config.in_devices.len());

                let mut client_midi_out_ports =
                    Vec::<jack::Port<jack::MidiOut>>::with_capacity(midi_config.out_devices.len());
                let mut client_midi_out_port_names =
                    Vec::<String>::with_capacity(midi_config.out_devices.len());
                let mut client_midi_out_connected_to =
                    Vec::<Option<String>>::with_capacity(midi_config.out_devices.len());
                let mut midi_out_port_info =
                    Vec::<StreamMidiDeviceInfo>::with_capacity(midi_config.out_devices.len());

                for (i, device_id) in midi_config.in_devices.iter().enumerate() {
                    if !system_midi_in_ports.contains(&device_id.name) {
                        if let MidiDeviceNotFoundBehavior::ReturnWithError =
                            options.error_behavior.midi_device_not_found
                        {
                            return Err(RunConfigError::MidiDeviceNotFound(device_id.name.clone()));
                        }
                        client_midi_in_connected_to.push(None);
                        midi_in_port_info
                            .push(StreamMidiDeviceInfo { id: device_id.clone(), success: false });
                    } else {
                        client_midi_in_connected_to.push(Some(device_id.name.clone()));
                        midi_in_port_info
                            .push(StreamMidiDeviceInfo { id: device_id.clone(), success: true });
                    }

                    let client_port_name = format!("in_{}", i + 1);
                    let client_port =
                        client.register_port(&client_port_name, jack::MidiIn::default())?;

                    client_midi_in_ports.push(client_port);
                    client_midi_in_port_names.push(client_port_name);
                }

                for (i, device_id) in midi_config.out_devices.iter().enumerate() {
                    if !system_midi_out_ports.contains(&device_id.name) {
                        if let MidiDeviceNotFoundBehavior::ReturnWithError =
                            options.error_behavior.midi_device_not_found
                        {
                            return Err(RunConfigError::MidiDeviceNotFound(device_id.name.clone()));
                        }
                        client_midi_out_connected_to.push(None);
                        midi_out_port_info
                            .push(StreamMidiDeviceInfo { id: device_id.clone(), success: false });
                    } else {
                        client_midi_out_connected_to.push(Some(device_id.name.clone()));
                        midi_out_port_info
                            .push(StreamMidiDeviceInfo { id: device_id.clone(), success: true });
                    }

                    let client_port_name = format!("out_{}", i + 1);
                    let client_port =
                        client.register_port(&client_port_name, jack::MidiOut::default())?;

                    client_midi_out_ports.push(client_port);
                    client_midi_out_port_names.push(client_port_name);
                }

                (
                    client_midi_in_ports,
                    client_midi_out_ports,
                    Some(MidiPortInfo {
                        client_midi_in_port_names,
                        client_midi_out_connected_to,
                        midi_in_port_info,
                        client_midi_out_port_names,
                        client_midi_in_connected_to,
                        midi_out_port_info,
                    }),
                )
            } else {
                (Vec::new(), Vec::new(), None)
            }
        } else {
            (Vec::new(), Vec::new(), None)
        }
    };

    let sample_rate = client.sample_rate() as u32;

    #[cfg(feature = "midi")]
    let midi_info = if let Some(midi_ports) = &midi_port_info {
        Some(MidiStreamInfo {
            midi_backend: MidiBackend::JackLinux,
            in_devices: midi_ports.midi_in_port_info.clone(),
            out_devices: midi_ports.midi_out_port_info.clone(),
            midi_buffer_size: options.midi_buffer_size as usize,
        })
    } else {
        None
    };

    let stream_info = StreamInfo {
        audio_backend: AudioBackend::JackLinux,
        audio_backend_version: None,
        audio_device: config.audio_device.clone(),
        audio_in_ports: audio_in_port_info,
        audio_out_ports: audio_out_port_info,
        sample_rate,
        buffer_size: StreamAudioBufferSize::FixedSized(client.buffer_size() as u32),
        estimated_latency: None,
        checking_for_silent_inputs: options.check_for_silent_inputs,
        #[cfg(feature = "midi")]
        midi_info,
    };

    process_handler.init(&stream_info);

    let process = JackProcessHandler::new(
        process_handler,
        client_audio_in_ports,
        client_audio_out_ports,
        #[cfg(feature = "midi")]
        client_midi_in_ports,
        #[cfg(feature = "midi")]
        client_midi_out_ports,
        &stream_info,
    );

    log::debug!("Activating Jack client...");

    // Activate the client, which starts the processing.
    let async_client = client.activate_async(
        JackNotificationHandler { error_handler: Some(error_handler), sample_rate },
        process,
    )?;

    // Try to automatically connect to system inputs/outputs.
    for (in_port, system_in_port) in
        client_audio_in_port_names.iter().zip(client_audio_in_connected_to.iter())
    {
        if let Some(system_in_port) = &system_in_port {
            if let Err(e) = async_client.as_client().connect_ports_by_name(system_in_port, in_port)
            {
                log::error!(
                    "Failed to connect jack audio ports src({}) dst({}): {}",
                    system_in_port,
                    in_port,
                    e
                );
                if let AudioPortNotFoundBehavior::ReturnWithError =
                    options.error_behavior.audio_port_not_found
                {
                    return Err(RunConfigError::AudioPortNotFound(in_port.clone()));
                }
            }
        }
    }
    for (out_port, system_out_port) in
        client_audio_out_port_names.iter().zip(client_audio_out_connected_to.iter())
    {
        if let Some(system_out_port) = &system_out_port {
            if let Err(e) =
                async_client.as_client().connect_ports_by_name(out_port, system_out_port)
            {
                log::error!(
                    "Failed to connect jack audio ports src({}) dst({}): {}",
                    out_port,
                    system_out_port,
                    e
                );
                if let AudioPortNotFoundBehavior::ReturnWithError =
                    options.error_behavior.audio_port_not_found
                {
                    return Err(RunConfigError::AudioPortNotFound(out_port.clone()));
                }
            }
        }
    }

    #[cfg(feature = "midi")]
    {
        if let Some(midi_ports) = &midi_port_info {
            for (in_port, system_in_port) in midi_ports
                .client_midi_in_port_names
                .iter()
                .zip(midi_ports.client_midi_in_connected_to.iter())
            {
                if let Some(system_in_port) = &system_in_port {
                    if let Err(e) =
                        async_client.as_client().connect_ports_by_name(system_in_port, in_port)
                    {
                        log::error!(
                            "Failed to connect jack midi ports src({}) dst({}): {}",
                            system_in_port,
                            in_port,
                            e
                        );
                        if let MidiDeviceNotFoundBehavior::ReturnWithError =
                            options.error_behavior.midi_device_not_found
                        {
                            return Err(RunConfigError::MidiDeviceNotFound(system_in_port.clone()));
                        }
                    }
                }
            }
            for (out_port, system_out_port) in midi_ports
                .client_midi_out_port_names
                .iter()
                .zip(midi_ports.client_midi_out_connected_to.iter())
            {
                if let Some(system_out_port) = &system_out_port {
                    if let Err(e) =
                        async_client.as_client().connect_ports_by_name(out_port, system_out_port)
                    {
                        log::error!(
                            "Failed to connect jack midi ports src({}) dst({}): {}",
                            out_port,
                            system_out_port,
                            e
                        );
                        if let MidiDeviceNotFoundBehavior::ReturnWithError =
                            options.error_behavior.midi_device_not_found
                        {
                            return Err(RunConfigError::MidiDeviceNotFound(
                                system_out_port.clone(),
                            ));
                        }
                    }
                }
            }
        }
    }

    Ok(StreamHandle { platform_handle: Box::new(JackStreamHandle { stream_info }) })
}

pub struct JackStreamHandle {
    stream_info: StreamInfo,
}

impl<P: ProcessHandler, E: ErrorHandler> PlatformStreamHandle<P, E> for JackStreamHandle {
    fn stream_info(&self) -> &StreamInfo {
        &self.stream_info
    }

    fn change_audio_port_config(
        &mut self,
        audio_in_ports: Option<Vec<String>>,
        audio_out_ports: Option<Vec<String>>,
    ) -> Result<(), ChangeAudioPortConfigError> {
        todo!()
    }

    fn change_audio_buffer_size_config(
        &mut self,
        config: AudioBufferSizeConfig,
    ) -> Result<(), ChangeAudioBufferSizeError> {
        todo!()
    }

    #[cfg(feature = "midi")]
    fn change_midi_device_config(
        &mut self,
        in_devices: Vec<DeviceID>,
        out_devices: Vec<DeviceID>,
    ) -> Result<(), ChangeMidiDeviceConfigError> {
        todo!()
    }

    fn can_change_audio_port_config(&self) -> bool {
        true
    }

    fn can_change_audio_buffer_size_config(&self) -> bool {
        false
    }

    #[cfg(feature = "midi")]
    fn can_change_midi_device_config(&self) -> bool {
        true
    }
}

struct JackProcessHandler<P: ProcessHandler> {
    process_handler: P,

    audio_in_ports: Vec<jack::Port<jack::AudioIn>>,
    audio_out_ports: Vec<jack::Port<jack::AudioOut>>,

    audio_in_buffers: Vec<Vec<f32>>,
    audio_out_buffers: Vec<Vec<f32>>,

    #[cfg(feature = "midi")]
    midi_in_ports: Vec<jack::Port<jack::MidiIn>>,
    #[cfg(feature = "midi")]
    midi_out_ports: Vec<jack::Port<jack::MidiOut>>,

    #[cfg(feature = "midi")]
    midi_in_buffers: Vec<MidiBuffer>,
    #[cfg(feature = "midi")]
    midi_out_buffers: Vec<MidiBuffer>,

    audio_buffer_size: usize,
    check_for_silence: bool,
    silent_audio_in_flags: Vec<bool>,
}

impl<P: ProcessHandler> JackProcessHandler<P> {
    fn new(
        process_handler: P,
        audio_in_ports: Vec<jack::Port<jack::AudioIn>>,
        audio_out_ports: Vec<jack::Port<jack::AudioOut>>,
        #[cfg(feature = "midi")] midi_in_ports: Vec<jack::Port<jack::MidiIn>>,
        #[cfg(feature = "midi")] midi_out_ports: Vec<jack::Port<jack::MidiOut>>,
        stream_info: &StreamInfo,
    ) -> Self {
        let audio_buffer_size = stream_info.buffer_size.max_buffer_size() as usize;

        let audio_in_buffers =
            (0..audio_in_ports.len()).map(|_| Vec::with_capacity(audio_buffer_size)).collect();
        let audio_out_buffers =
            (0..audio_out_ports.len()).map(|_| Vec::with_capacity(audio_buffer_size)).collect();

        let silent_audio_in_flags = vec![false; audio_in_ports.len()];

        #[cfg(feature = "midi")]
        let (midi_in_buffers, midi_out_buffers) = {
            if let Some(midi_info) = &stream_info.midi_info {
                let midi_buffer_size = midi_info.midi_buffer_size;

                (
                    (0..midi_in_ports.len()).map(|_| MidiBuffer::new(midi_buffer_size)).collect(),
                    (0..midi_out_ports.len()).map(|_| MidiBuffer::new(midi_buffer_size)).collect(),
                )
            } else {
                (Vec::new(), Vec::new())
            }
        };

        Self {
            process_handler,
            audio_in_ports,
            audio_out_ports,
            audio_in_buffers,
            audio_out_buffers,
            #[cfg(feature = "midi")]
            midi_in_ports,
            #[cfg(feature = "midi")]
            midi_out_ports,
            #[cfg(feature = "midi")]
            midi_in_buffers,
            #[cfg(feature = "midi")]
            midi_out_buffers,
            audio_buffer_size: audio_buffer_size as usize,
            check_for_silence: stream_info.checking_for_silent_inputs,
            silent_audio_in_flags,
        }
    }
}

impl<P: ProcessHandler> jack::ProcessHandler for JackProcessHandler<P> {
    fn process(&mut self, _: &jack::Client, ps: &jack::ProcessScope) -> jack::Control {
        let mut frames: usize = 0;

        // Copy audio inputs
        for (buffer, port) in self.audio_in_buffers.iter_mut().zip(self.audio_in_ports.iter()) {
            let port_buffer = port.as_slice(ps);

            // Sanity checks.
            if port_buffer.len() > self.audio_buffer_size {
                log::warn!(
                    "Jack sent a buffer size of {} when the max buffer size is {}",
                    port_buffer.len(),
                    self.audio_buffer_size
                );
            }
            if frames != 0 && port_buffer.len() != frames {
                log::error!(
                    "Jack sent buffers of unmatched length: {}, {}",
                    frames,
                    port_buffer.len()
                );
                frames = port_buffer.len().min(frames);
            } else {
                frames = port_buffer.len()
            }

            buffer.resize(port_buffer.len(), 0.0);
            buffer.copy_from_slice(&port_buffer);
        }

        if self.audio_in_buffers.len() == 0 {
            // Check outputs for number of frames instead.
            if let Some(out_port) = self.audio_out_ports.first_mut() {
                frames = out_port.as_mut_slice(ps).len();
            }
        }

        // Clear audio outputs.
        for buffer in self.audio_out_buffers.iter_mut() {
            buffer.clear();
            buffer.resize(frames, 0.0);
        }

        #[cfg(feature = "midi")]
        {
            // Collect MIDI inputs
            for (midi_buffer, port) in
                self.midi_in_buffers.iter_mut().zip(self.midi_in_ports.iter())
            {
                midi_buffer.clear();

                for event in port.iter(ps) {
                    if let Err(e) = midi_buffer.push_raw(event.time, event.bytes) {
                        match e {
                            MidiBufferPushError::BufferFull => {
                                log::error!("Midi event dropped because buffer is full!");
                            }
                            MidiBufferPushError::EventTooLong(_) => {
                                log::debug!(
                                    "Midi event {:?} was dropped because it is too long",
                                    event.bytes
                                );
                            }
                        }
                    }
                }
            }

            // Clear MIDI outputs
            for midi_buffer in self.midi_out_buffers.iter_mut() {
                midi_buffer.clear();
            }
        }

        if self.check_for_silence {
            // TODO: This could probably be optimized.
            for (buffer, flag) in
                self.audio_in_buffers.iter().zip(self.silent_audio_in_flags.iter_mut())
            {
                *flag = true;
                for smp in buffer.iter() {
                    if *smp != 0.0 {
                        *flag = false;
                        break;
                    }
                }
            }
        }

        self.process_handler.process(ProcessInfo {
            audio_inputs: &self.audio_in_buffers,
            audio_outputs: &mut self.audio_out_buffers,
            frames,
            silent_audio_inputs: &self.silent_audio_in_flags,
            #[cfg(feature = "midi")]
            midi_inputs: &self.midi_in_buffers,
            #[cfg(feature = "midi")]
            midi_outputs: &mut self.midi_out_buffers,
        });

        // Copy processed data to audio outputs
        for (buffer, port) in self.audio_out_buffers.iter().zip(self.audio_out_ports.iter_mut()) {
            let port_buffer = port.as_mut_slice(ps);

            // Sanity check
            let mut len = port_buffer.len();
            if port_buffer.len() != buffer.len() {
                log::error!(
                    "Jack sent buffers of unmatched length: {}, {}",
                    port_buffer.len(),
                    buffer.len()
                );
                len = port_buffer.len().min(buffer.len());
            }

            port_buffer[0..len].copy_from_slice(&buffer[0..len]);
        }

        #[cfg(feature = "midi")]
        {
            // Copy processed data to MIDI outputs
            for (midi_buffer, port) in
                self.midi_out_buffers.iter().zip(self.midi_out_ports.iter_mut())
            {
                let mut port_writer = port.writer(ps);

                for event in midi_buffer.events() {
                    if let Err(e) = port_writer
                        .write(&jack::RawMidi { time: event.delta_frames, bytes: &event.data() })
                    {
                        log::error!("Warning: Could not copy midi data to Jack output: {}", e);
                    }
                }
            }
        }

        jack::Control::Continue
    }
}

struct JackNotificationHandler<E: ErrorHandler> {
    error_handler: Option<E>,

    sample_rate: u32,
}

impl<E: ErrorHandler> jack::NotificationHandler for JackNotificationHandler<E> {
    fn thread_init(&self, _: &jack::Client) {
        log::debug!("JACK: thread init");
    }

    fn shutdown(&mut self, status: jack::ClientStatus, reason: &str) {
        let msg = format!("JACK: shutdown with status {:?} because \"{}\"", status, reason);

        log::error!("{}", msg);

        if let Some(error_handler) = self.error_handler.take() {
            error_handler.fatal_error(FatalStreamError::AudioServerShutdown { msg: Some(msg) });
        }
    }

    fn freewheel(&mut self, _: &jack::Client, is_enabled: bool) {
        log::debug!("JACK: freewheel mode is {}", if is_enabled { "on" } else { "off" });
    }

    fn sample_rate(&mut self, _: &jack::Client, srate: jack::Frames) -> jack::Control {
        log::debug!("JACK: sample rate changed to {}", srate);

        // Why does Jack allow changing the samplerate mid-stream?!
        // Just shut down the audio thread in this case.
        if srate != self.sample_rate {
            if let Some(error_handler) = self.error_handler.take() {
                error_handler.fatal_error(FatalStreamError::AudioServerChangedSamplerate(srate));
                return jack::Control::Quit;
            }
        }

        jack::Control::Continue
    }

    fn client_registration(&mut self, _: &jack::Client, name: &str, is_reg: bool) {
        log::debug!(
            "JACK: {} client with name \"{}\"",
            if is_reg { "registered" } else { "unregistered" },
            name
        );
    }

    fn port_registration(&mut self, _: &jack::Client, port_id: jack::PortId, is_reg: bool) {
        log::debug!(
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
        log::debug!("JACK: port with id {} renamed from {} to {}", port_id, old_name, new_name);
        jack::Control::Continue
    }

    fn ports_connected(
        &mut self,
        _: &jack::Client,
        port_id_a: jack::PortId,
        port_id_b: jack::PortId,
        are_connected: bool,
    ) {
        log::debug!(
            "JACK: ports with id {} and {} are {}",
            port_id_a,
            port_id_b,
            if are_connected { "connected" } else { "disconnected" }
        );
    }

    fn graph_reorder(&mut self, _: &jack::Client) -> jack::Control {
        log::debug!("JACK: graph reordered");
        jack::Control::Continue
    }

    fn xrun(&mut self, _: &jack::Client) -> jack::Control {
        //log::warn!("JACK: xrun occurred");
        jack::Control::Continue
    }
}

impl From<jack::Error> for RunConfigError {
    fn from(e: jack::Error) -> Self {
        RunConfigError::PlatformSpecific(Box::new(e))
    }
}
