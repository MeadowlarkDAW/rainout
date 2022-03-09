use ringbuf::Producer;

use crate::error::{ChangeAudioChannelsError, ChangeBlockSizeError, RunConfigError};
use crate::{
    AudioBufferStreamInfo, AudioChannelStreamInfo, AudioDeviceStreamInfo, AutoOption, Backend,
    DeviceID, PlatformStreamHandle, ProcessHandler, RainoutConfig, RunOptions, StreamHandle,
    StreamInfo, StreamMsg,
};

#[cfg(feature = "midi")]
use crate::{
    error::ChangeMidiPortsError, MidiControlScheme, MidiPortConfig, MidiPortStreamInfo,
    MidiStreamInfo,
};

use super::{JackNotificationHandler, JackProcessHandler, DUMMY_CLIENT_NAME, JACK_DEVICE_NAME};

const DEFAULT_CLIENT_NAME: &'static str = "rustydaw_io_client";

pub fn estimated_sample_rate_and_latency(
    _config: &RainoutConfig,
) -> Result<(Option<u32>, Option<u32>), RunConfigError> {
    match jack::Client::new(DUMMY_CLIENT_NAME, jack::ClientOptions::empty()) {
        Ok((client, _status)) => {
            Ok((Some(client.sample_rate() as u32), Some(client.buffer_size())))
        }
        Err(e) => match e {
            jack::Error::LoadLibraryError(e) => {
                log::warn!("Jack server is not installed: {}", e);

                Err(RunConfigError::AudioBackendNotInstalled(Backend::Jack))
            }
            e => {
                log::warn!("Jack server is unavailable: {}", e);

                Err(RunConfigError::AudioBackendNotRunning(Backend::Jack))
            }
        },
    }
}

pub fn run<P: ProcessHandler>(
    config: &RainoutConfig,
    options: &RunOptions,
    mut process_handler: P,
) -> Result<StreamHandle<P>, RunConfigError> {
    // --- Create Jack client -----------------------------------------------------------------------

    log::debug!("Creating Jack client...");

    let client_name =
        options.use_application_name.clone().unwrap_or(String::from(DEFAULT_CLIENT_NAME));

    log::debug!("Registering Jack client with name {}", &client_name);

    let (client, _status) = jack::Client::new(&client_name, jack::ClientOptions::empty())?;

    // --- Find system audio ports ------------------------------------------------------------------

    let system_audio_in_ports: Vec<String> =
        client.ports(None, Some("32 bit float mono audio"), jack::PortFlags::IS_OUTPUT);
    let system_audio_out_ports: Vec<String> =
        client.ports(None, Some("32 bit float mono audio"), jack::PortFlags::IS_INPUT);

    // --- Register client audio ports --------------------------------------------------------------

    let use_audio_in_ports = match &config.jack_in_ports {
        AutoOption::Use(ports) => ports.clone(),
        AutoOption::Auto => {
            let mut use_ports: Vec<String> = Vec::new();

            if options.auto_audio_inputs {
                if !system_audio_in_ports.is_empty() {
                    // Find index of default input port.
                    let mut default_in_port = 0; // Fallback to first available port.
                    for (i, port) in system_audio_in_ports.iter().enumerate() {
                        if port == "system:capture_1" {
                            default_in_port = i;
                            break;
                        }
                    }

                    use_ports.push(system_audio_in_ports[default_in_port].clone());
                }
            }

            use_ports
        }
    };

    let use_audio_out_ports = match &config.jack_out_ports {
        AutoOption::Use(ports) => {
            if options.must_have_stereo_output && ports.len() < 2 {
                return Err(RunConfigError::ConfigHasNoStereoOutput);
            }

            ports.clone()
        }
        AutoOption::Auto => {
            let mut use_ports: Vec<String> = Vec::new();

            if options.must_have_stereo_output && system_audio_out_ports.len() < 2 {
                return Err(RunConfigError::AutoNoStereoOutputFound);
            }

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

            if !system_audio_out_ports.is_empty() {
                if system_audio_in_ports.len() == 1
                    || default_out_port_left == default_out_port_right
                {
                    use_ports.push(system_audio_in_ports[default_out_port_left].clone());
                } else {
                    use_ports.push(system_audio_in_ports[default_out_port_left].clone());
                    use_ports.push(system_audio_in_ports[default_out_port_right].clone());
                }
            }

            use_ports
        }
    };

    let mut client_audio_in_ports =
        Vec::<jack::Port<jack::AudioIn>>::with_capacity(use_audio_in_ports.len());
    let mut client_audio_in_port_names = Vec::<String>::with_capacity(use_audio_in_ports.len());
    let mut client_audio_in_connected_to =
        Vec::<Option<String>>::with_capacity(use_audio_in_ports.len());
    let mut audio_in_channel_info =
        Vec::<AudioChannelStreamInfo>::with_capacity(use_audio_in_ports.len());

    let mut client_audio_out_ports =
        Vec::<jack::Port<jack::AudioOut>>::with_capacity(use_audio_out_ports.len());
    let mut client_audio_out_port_names = Vec::<String>::with_capacity(use_audio_out_ports.len());
    let mut client_audio_out_connected_to =
        Vec::<Option<String>>::with_capacity(use_audio_out_ports.len());
    let mut audio_out_channel_info =
        Vec::<AudioChannelStreamInfo>::with_capacity(use_audio_out_ports.len());

    for (i, port) in use_audio_in_ports.iter().enumerate() {
        if !system_audio_in_ports.contains(port) {
            if !options.empty_buffers_for_failed_ports {
                return Err(RunConfigError::AudioPortNotFound(port.clone()));
            }
            client_audio_in_connected_to.push(None);
            audio_in_channel_info.push(AudioChannelStreamInfo {
                connected_to_index: 0,
                connected_to_name: None,
                connected_to_system: false,
            });
        } else {
            client_audio_in_connected_to.push(Some(port.clone()));
            audio_in_channel_info.push(AudioChannelStreamInfo {
                connected_to_index: 0,
                connected_to_name: Some(port.clone()),
                connected_to_system: true,
            });
        }

        let client_port_name = format!("in_{}", i + 1);
        let client_port = client.register_port(&client_port_name, jack::AudioIn::default())?;

        client_audio_in_ports.push(client_port);
        client_audio_in_port_names.push(client_port_name);
    }

    for (i, port) in use_audio_out_ports.iter().enumerate() {
        if !system_audio_out_ports.contains(port) {
            if !options.empty_buffers_for_failed_ports {
                return Err(RunConfigError::AudioPortNotFound(port.clone()));
            }
            client_audio_out_connected_to.push(None);
            audio_out_channel_info.push(AudioChannelStreamInfo {
                connected_to_index: 0,
                connected_to_name: None,
                connected_to_system: false,
            });
        } else {
            client_audio_out_connected_to.push(Some(port.clone()));
            audio_out_channel_info.push(AudioChannelStreamInfo {
                connected_to_index: 0,
                connected_to_name: Some(port.clone()),
                connected_to_system: true,
            });
        }

        let client_port_name = format!("out_{}", i + 1);
        let client_port = client.register_port(&client_port_name, jack::AudioOut::default())?;

        client_audio_out_ports.push(client_port);
        client_audio_out_port_names.push(client_port_name);
    }

    // --- Register client MIDI ports ---------------------------------------------------------------

    #[cfg(feature = "midi")]
    struct MidiPortInfo {
        client_midi_in_port_names: Vec<String>,
        client_midi_in_connected_to: Vec<Option<String>>,
        midi_in_port_info: Vec<MidiPortStreamInfo>,

        client_midi_out_port_names: Vec<String>,
        client_midi_out_connected_to: Vec<Option<String>>,
        midi_out_port_info: Vec<MidiPortStreamInfo>,
    }

    #[cfg(feature = "midi")]
    let (client_midi_in_ports, client_midi_out_ports, midi_port_info) = {
        if let Some(midi_config) = &config.midi_config {
            let use_jack_midi = match &midi_config.midi_backend {
                AutoOption::Auto => true,
                AutoOption::Use(b) => {
                    if let Backend::Jack = b {
                        true
                    } else {
                        false
                    }
                }
            };

            if use_jack_midi {
                // Find system MIDI ports
                let system_midi_in_ports: Vec<String> =
                    client.ports(None, Some("8 bit raw midi"), jack::PortFlags::IS_OUTPUT);
                let system_midi_out_ports: Vec<String> =
                    client.ports(None, Some("8 bit raw midi"), jack::PortFlags::IS_INPUT);

                let use_midi_in_ports = match &midi_config.in_device_ports {
                    AutoOption::Use(ports) => ports.clone(),
                    AutoOption::Auto => {
                        let mut use_ports: Vec<MidiPortConfig> = Vec::new();

                        if !system_midi_in_ports.is_empty() {
                            // Find index of default input port.
                            let mut default_in_port = 0; // Fallback to first available port.
                            for (i, port) in system_midi_in_ports.iter().enumerate() {
                                // "system:midi_capture_1" is usually Jack's built-in `Midi-Through` device.
                                // What we usually want is first available port of the user's hardware MIDI
                                // controller, which is commonly mapped to "system:midi_capture_2".
                                if port == "system:midi_capture_2" {
                                    default_in_port = i;
                                    break;
                                }
                            }

                            use_ports.push(MidiPortConfig {
                                device_id: DeviceID {
                                    name: system_audio_in_ports[default_in_port].clone(),
                                    identifier: None,
                                },
                                port_index: 0,
                                control_scheme: MidiControlScheme::Midi1,
                            });
                        }

                        use_ports
                    }
                };

                let use_midi_out_ports = match &midi_config.out_device_ports {
                    AutoOption::Use(ports) => ports.clone(),
                    AutoOption::Auto => {
                        let mut use_ports: Vec<MidiPortConfig> = Vec::new();

                        if !system_midi_out_ports.is_empty() {
                            // Find index of default input port.
                            let mut default_out_port = 0; // Fallback to first available port.
                            for (i, port) in system_midi_out_ports.iter().enumerate() {
                                // "system:midi_playback_1" is usually Jack's built-in `Midi-Through` device.
                                // What we usually want is first available port of the user's hardware MIDI
                                // controller, which is commonly mapped to "system:midi_playback_2".
                                if port == "system:midi_playback_2" {
                                    default_out_port = i;
                                    break;
                                }
                            }

                            use_ports.push(MidiPortConfig {
                                device_id: DeviceID {
                                    name: system_audio_in_ports[default_out_port].clone(),
                                    identifier: None,
                                },
                                port_index: 0,
                                control_scheme: MidiControlScheme::Midi1,
                            });
                        }

                        use_ports
                    }
                };

                let mut client_midi_in_ports =
                    Vec::<jack::Port<jack::MidiIn>>::with_capacity(use_midi_in_ports.len());
                let mut client_midi_in_port_names =
                    Vec::<String>::with_capacity(use_midi_in_ports.len());
                let mut client_midi_in_connected_to =
                    Vec::<Option<String>>::with_capacity(use_midi_in_ports.len());
                let mut midi_in_port_info =
                    Vec::<MidiPortStreamInfo>::with_capacity(use_midi_in_ports.len());

                let mut client_midi_out_ports =
                    Vec::<jack::Port<jack::MidiOut>>::with_capacity(use_midi_out_ports.len());
                let mut client_midi_out_port_names =
                    Vec::<String>::with_capacity(use_midi_out_ports.len());
                let mut client_midi_out_connected_to =
                    Vec::<Option<String>>::with_capacity(use_midi_out_ports.len());
                let mut midi_out_port_info =
                    Vec::<MidiPortStreamInfo>::with_capacity(use_midi_out_ports.len());

                for (i, port_config) in use_midi_in_ports.iter().enumerate() {
                    if !system_midi_in_ports.contains(&port_config.device_id.name) {
                        if !options.empty_buffers_for_failed_ports {
                            return Err(RunConfigError::MidiDeviceNotFound(
                                port_config.device_id.name.clone(),
                            ));
                        }
                        client_midi_in_connected_to.push(None);
                        midi_in_port_info.push(MidiPortStreamInfo {
                            id: port_config.device_id.clone(),
                            port_index: 0,
                            control_scheme: MidiControlScheme::Midi1,
                            connected_to_system: false,
                        });
                    } else {
                        client_midi_in_connected_to.push(Some(port_config.device_id.name.clone()));
                        midi_in_port_info.push(MidiPortStreamInfo {
                            id: port_config.device_id.clone(),
                            port_index: 0,
                            control_scheme: MidiControlScheme::Midi1,
                            connected_to_system: true,
                        });
                    }

                    let client_port_name = format!("in_{}", i + 1);
                    let client_port =
                        client.register_port(&client_port_name, jack::MidiIn::default())?;

                    client_midi_in_ports.push(client_port);
                    client_midi_in_port_names.push(client_port_name);
                }

                for (i, port_config) in use_midi_out_ports.iter().enumerate() {
                    if !system_midi_out_ports.contains(&port_config.device_id.name) {
                        if !options.empty_buffers_for_failed_ports {
                            return Err(RunConfigError::MidiDeviceNotFound(
                                port_config.device_id.name.clone(),
                            ));
                        }
                        client_midi_out_connected_to.push(None);
                        midi_out_port_info.push(MidiPortStreamInfo {
                            id: port_config.device_id.clone(),
                            port_index: 0,
                            control_scheme: MidiControlScheme::Midi1,
                            connected_to_system: false,
                        });
                    } else {
                        client_midi_out_connected_to.push(Some(port_config.device_id.name.clone()));
                        midi_out_port_info.push(MidiPortStreamInfo {
                            id: port_config.device_id.clone(),
                            port_index: 0,
                            control_scheme: MidiControlScheme::Midi1,
                            connected_to_system: true,
                        });
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

    // --- Construct stream info ----------------------------------------------------------------------

    let sample_rate = client.sample_rate() as u32;

    #[cfg(feature = "midi")]
    let midi_info = if let Some(midi_ports) = &midi_port_info {
        Some(MidiStreamInfo {
            midi_backend: Backend::Jack,
            in_ports: midi_ports.midi_in_port_info.clone(),
            out_ports: midi_ports.midi_out_port_info.clone(),
            midi_buffer_size: options.midi_buffer_size as usize,
        })
    } else {
        None
    };

    let stream_info = StreamInfo {
        audio_backend: Backend::Jack,
        audio_backend_version: None,
        audio_device: AudioDeviceStreamInfo::Single(DeviceID {
            name: JACK_DEVICE_NAME.to_string(),
            identifier: None,
        }),
        audio_in_channels: audio_in_channel_info,
        audio_out_channels: audio_out_channel_info,
        sample_rate,
        buffer_size: AudioBufferStreamInfo::FixedSized(client.buffer_size() as u32),
        estimated_latency: None,
        checking_for_silent_inputs: options.check_for_silent_inputs,
        #[cfg(feature = "midi")]
        midi_info,
    };

    // Pass stream info to client for initialization.
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

    // --- Spawn Jack stream -----------------------------------------------------------------------

    let (to_stream_handle_tx, from_audio_thread_rx) =
        ringbuf::RingBuffer::new(options.msg_buffer_size).split();

    log::debug!("Activating Jack client...");

    // Activate the client, which starts the processing.
    let async_client = client
        .activate_async(JackNotificationHandler::new(to_stream_handle_tx, sample_rate), process)?;

    // --- Connect system audio ports to client ports ----------------------------------------------

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
                if !options.empty_buffers_for_failed_ports {
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
                if !options.empty_buffers_for_failed_ports {
                    return Err(RunConfigError::AudioPortNotFound(out_port.clone()));
                }
            }
        }
    }

    // --- Connect system MIDI ports to client ports -----------------------------------------------

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
                        if !options.empty_buffers_for_failed_ports {
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
                        if !options.empty_buffers_for_failed_ports {
                            return Err(RunConfigError::MidiDeviceNotFound(
                                system_out_port.clone(),
                            ));
                        }
                    }
                }
            }
        }
    }

    Ok(StreamHandle {
        platform_handle: Box::new(JackStreamHandle { stream_info }),
        messages: from_audio_thread_rx,
    })
}

pub(crate) fn push_stream_msg(to_stream_handle_tx: &mut Producer<StreamMsg>, msg: StreamMsg) {
    if let Err(e) = to_stream_handle_tx.push(msg) {
        log::error!("Failed to send stream message {:?}: message buffer is full!", e);
    }
}

pub struct JackStreamHandle {
    stream_info: StreamInfo,
}

impl<P: ProcessHandler> PlatformStreamHandle<P> for JackStreamHandle {
    fn stream_info(&self) -> &StreamInfo {
        &self.stream_info
    }

    fn change_audio_channels(
        &mut self,
        _audio_in_ports: Vec<usize>,
        _audio_out_ports: Vec<usize>,
    ) -> Result<(), ChangeAudioChannelsError> {
        Err(ChangeAudioChannelsError::JackMustUsePortNames)
    }

    fn change_jack_audio_ports(
        &mut self,
        in_port_names: Vec<String>,
        out_port_names: Vec<String>,
    ) -> Result<(), ChangeAudioChannelsError> {
        todo!()
    }

    fn change_block_size(&mut self, _block_size: u32) -> Result<(), ChangeBlockSizeError> {
        Err(ChangeBlockSizeError::NotSupportedByBackend)
    }

    #[cfg(feature = "midi")]
    fn change_midi_ports(
        &mut self,
        in_devices: Vec<MidiPortConfig>,
        out_devices: Vec<MidiPortConfig>,
    ) -> Result<(), ChangeMidiPortsError> {
        todo!()
    }

    fn can_change_audio_channels(&self) -> bool {
        true
    }

    fn can_change_block_size(&self) -> bool {
        false
    }

    #[cfg(feature = "midi")]
    fn can_change_midi_ports(&self) -> bool {
        true
    }
}

impl From<jack::Error> for RunConfigError {
    fn from(e: jack::Error) -> Self {
        RunConfigError::PlatformSpecific(Box::new(e))
    }
}
