use crate::error::JackEnumerationError;
use crate::{
    AudioBackendOptions, AudioDeviceOptions, Backend, BackendStatus, ChannelLayout, DeviceID,
    JackAudioDeviceOptions, MidiBackendOptions, MidiPortOptions,
};

use super::DUMMY_CLIENT_NAME;

pub fn enumerate_audio_backend() -> AudioBackendOptions {
    log::debug!("Enumerating Jack server...");

    match jack::Client::new(DUMMY_CLIENT_NAME, jack::ClientOptions::empty()) {
        Ok((_client, _status)) => {
            log::debug!("Jack server is running");

            AudioBackendOptions {
                backend: Backend::Jack,
                version: None,
                status: BackendStatus::Running,
                device_options: Some(AudioDeviceOptions::JackSystemWideDevice),
            }
        }
        Err(e) => match e {
            jack::Error::LoadLibraryError(e) => {
                log::warn!("Jack server is not installed: {}", e);

                AudioBackendOptions {
                    backend: Backend::Jack,
                    version: None,
                    status: BackendStatus::NotInstalled,
                    device_options: None,
                }
            }
            e => {
                log::warn!("Jack server is unavailable: {}", e);

                AudioBackendOptions {
                    backend: Backend::Jack,
                    version: None,
                    status: BackendStatus::NotRunning,
                    device_options: None,
                }
            }
        },
    }
}

pub fn enumerate_audio_device() -> Result<JackAudioDeviceOptions, JackEnumerationError> {
    log::debug!("Enumerating Jack audio device...");

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
            let default_input_ports = if !system_audio_in_ports.is_empty() {
                Some((vec![default_in_port], ChannelLayout::Mono))
            } else {
                None
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
            let default_output_ports = if !system_audio_out_ports.is_empty() {
                if system_audio_in_ports.len() == 1
                    || default_out_port_left == default_out_port_right
                {
                    Some((vec![default_out_port_left], ChannelLayout::Mono))
                } else {
                    Some((
                        vec![default_out_port_left, default_out_port_right],
                        ChannelLayout::Stereo,
                    ))
                }
            } else {
                None
            };

            // Only one sample rate is available which is the sample rate configured
            // for the server.
            let sample_rate = client.sample_rate() as u32;

            // Only one fixed buffer size is available which is the buffer size
            // configured for the server.
            let block_size = client.buffer_size() as u32;

            Ok(JackAudioDeviceOptions {
                sample_rate,
                block_size,
                input_ports: system_audio_in_ports,
                output_ports: system_audio_out_ports,
                default_input_ports,
                default_output_ports,
            })
        }
        Err(e) => match e {
            jack::Error::LoadLibraryError(e) => {
                log::warn!("Jack server is not installed: {}", e);

                Err(JackEnumerationError::NotInstalled)
            }
            e => {
                log::warn!("Jack server is unavailable: {}", e);

                Err(JackEnumerationError::NotRunning)
            }
        },
    }
}

#[cfg(feature = "midi")]
pub fn enumerate_midi_backend() -> MidiBackendOptions {
    use crate::MidiControlScheme;

    log::debug!("Enumerating Jack MIDI server...");

    match jack::Client::new(DUMMY_CLIENT_NAME, jack::ClientOptions::empty()) {
        Ok((client, _status)) => {
            let in_device_ports: Vec<MidiPortOptions> = client
                .ports(None, Some("8 bit raw midi"), jack::PortFlags::IS_OUTPUT)
                .drain(..)
                .map(|n| MidiPortOptions {
                    id: DeviceID { name: n, identifier: None },
                    port_index: 0,
                    control_type: MidiControlScheme::Midi1,
                })
                .collect();
            let out_device_ports: Vec<MidiPortOptions> = client
                .ports(None, Some("8 bit raw midi"), jack::PortFlags::IS_INPUT)
                .drain(..)
                .map(|n| MidiPortOptions {
                    id: DeviceID { name: n, identifier: None },
                    port_index: 0,
                    control_type: MidiControlScheme::Midi1,
                })
                .collect();

            // Find index of the default in port.
            let mut default_in_port = 0; // Fallback to first available port.
            for (i, device) in in_device_ports.iter().enumerate() {
                // "system:midi_capture_1" is usually Jack's built-in `Midi-Through` device.
                // What we usually want is first available port of the user's hardware MIDI controller, which is
                // commonly mapped to "system:midi_capture_2".
                if &device.id.name == "system:midi_capture_2" {
                    default_in_port = i;
                    break;
                }
            }
            let default_in_port =
                if in_device_ports.is_empty() { None } else { Some(default_in_port) };

            // Find index of the default out port.
            let mut default_out_port = 0; // Fallback to first available port.
            for (i, device) in out_device_ports.iter().enumerate() {
                // "system:midi_capture_1" is usually Jack's built-in `Midi-Through` device.
                // What we usually want is first available port of the user's hardware MIDI controller, which is
                // commonly mapped to "system:midi_playback_2".
                if &device.id.name == "system:midi_playback_2" {
                    default_out_port = i;
                    break;
                }
            }
            let default_out_port =
                if out_device_ports.is_empty() { None } else { Some(default_out_port) };

            MidiBackendOptions {
                backend: Backend::Jack,
                version: None,
                status: BackendStatus::Running,
                in_device_ports,
                out_device_ports,
                default_in_port,
                default_out_port,
            }
        }
        Err(e) => match e {
            jack::Error::LoadLibraryError(e) => {
                log::warn!("Jack server is not installed: {}", e);

                MidiBackendOptions {
                    backend: Backend::Jack,
                    version: None,
                    status: BackendStatus::NotInstalled,
                    in_device_ports: Vec::new(),
                    out_device_ports: Vec::new(),
                    default_in_port: None,
                    default_out_port: None,
                }
            }
            e => {
                log::warn!("Jack server is unavailable: {}", e);

                MidiBackendOptions {
                    backend: Backend::Jack,
                    version: None,
                    status: BackendStatus::NotRunning,
                    in_device_ports: Vec::new(),
                    out_device_ports: Vec::new(),
                    default_in_port: None,
                    default_out_port: None,
                }
            }
        },
    }
}
