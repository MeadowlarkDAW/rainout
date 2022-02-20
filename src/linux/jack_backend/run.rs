use ringbuf::Producer;

use crate::error::{ChangeAudioBufferSizeError, ChangeAudioPortConfigError, RunConfigError};
use crate::error_behavior::AudioPortNotFoundBehavior;
use crate::{
    AudioBackend, AudioBufferSizeConfig, Config, DeviceID, PlatformStreamHandle, ProcessHandler,
    RunOptions, StreamAudioBufferSize, StreamAudioPortInfo, StreamHandle, StreamInfo, StreamMsg,
    StreamMsgChannel,
};

#[cfg(feature = "midi")]
use crate::{
    error::ChangeMidiDeviceConfigError, error_behavior::MidiDeviceNotFoundBehavior, MidiBackend,
    MidiStreamInfo, StreamMidiDeviceInfo,
};

use super::{JackNotificationHandler, JackProcessHandler};

const DEFAULT_CLIENT_NAME: &'static str = "rustydaw_io_client";

pub fn run<P: ProcessHandler>(
    config: &Config,
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

    let (msg_channel, mut to_msg_channel_tx) = StreamMsgChannel::new(options.msg_buffer_size);

    for (i, port) in config.audio_in_ports.iter().enumerate() {
        if !system_audio_in_ports.contains(port) {
            if let AudioPortNotFoundBehavior::ReturnWithError =
                options.error_behavior.audio_port_not_found
            {
                return Err(RunConfigError::AudioPortNotFound(port.clone()));
            }
            push_stream_msg(&mut to_msg_channel_tx, StreamMsg::AudioInPortNotFound(port.clone()));
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
            push_stream_msg(&mut to_msg_channel_tx, StreamMsg::AudioOutPortNotFound(port.clone()));
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

    // --- Register client MIDI ports ---------------------------------------------------------------

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
                // Find system MIDI ports
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
                        push_stream_msg(
                            &mut to_msg_channel_tx,
                            StreamMsg::MidiInDeviceNotFound(device_id.name.clone()),
                        );
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
                        push_stream_msg(
                            &mut to_msg_channel_tx,
                            StreamMsg::MidiOutDeviceNotFound(device_id.name.clone()),
                        );
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

    // --- Construct stream info ----------------------------------------------------------------------

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

    log::debug!("Activating Jack client...");

    // Activate the client, which starts the processing.
    let async_client = client
        .activate_async(JackNotificationHandler::new(to_msg_channel_tx, sample_rate), process)?;

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

    Ok(StreamHandle {
        platform_handle: Box::new(JackStreamHandle { stream_info }),
        messages: msg_channel,
    })
}

pub(crate) fn push_stream_msg(to_msg_channel_tx: &mut Producer<StreamMsg>, msg: StreamMsg) {
    if let Err(e) = to_msg_channel_tx.push(msg) {
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

impl From<jack::Error> for RunConfigError {
    fn from(e: jack::Error) -> Self {
        RunConfigError::PlatformSpecific(Box::new(e))
    }
}
