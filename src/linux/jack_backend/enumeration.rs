use crate::{
    AudioBackend, AudioBackendInfo, AudioBackendStatus, AudioBufferSizeInfo, AudioDeviceInfo,
    ChannelLayout, DefaultChannelLayout, DeviceID, MidiBackendStatus, SampleRateInfo,
};

#[cfg(feature = "midi")]
use crate::{MidiBackend, MidiBackendInfo, MidiDeviceInfo};

const DUMMY_CLIENT_NAME: &'static str = "rustydaw_io_dummy_client";
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
                DefaultChannelLayout {
                    layout: ChannelLayout::Mono,
                    device_ports: vec![default_in_port],
                }
            } else {
                DefaultChannelLayout::empty()
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
                    DefaultChannelLayout {
                        layout: ChannelLayout::Mono,
                        device_ports: vec![default_out_port_left],
                    }
                } else {
                    DefaultChannelLayout {
                        layout: ChannelLayout::Stereo,
                        device_ports: vec![default_out_port_left, default_out_port_right],
                    }
                }
            } else {
                DefaultChannelLayout::empty()
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
                sample_rates: SampleRateInfo::Unconfigurable(sample_rate),
                buffer_sizes: AudioBufferSizeInfo::UnconfigurableFixed(buffer_size),
                default_input_layout,
                default_output_layout,
            };

            return AudioBackendInfo {
                backend: AudioBackend::JackLinux,
                version: None,
                status: AudioBackendStatus::RunningWithSystemWideDevice(device),
            };
        }
        Err(e) => {
            log::warn!("Jack server is unavailable: {}", e);
        }
    }

    AudioBackendInfo {
        backend: AudioBackend::JackLinux,
        version: None,
        status: AudioBackendStatus::NotRunning,
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
            let default_in_i = if in_devices.is_empty() { None } else { Some(default_in_port) };

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
            let default_out_i = if out_devices.is_empty() { None } else { Some(default_out_port) };

            if in_devices.is_empty() && out_devices.is_empty() {
                return MidiBackendInfo {
                    backend: MidiBackend::JackLinux,
                    version: None,
                    status: MidiBackendStatus::RunningButNoDevices,
                };
            }

            return MidiBackendInfo {
                backend: MidiBackend::JackLinux,
                version: None,
                status: MidiBackendStatus::Running {
                    in_devices,
                    out_devices,
                    default_in_i,
                    default_out_i,
                },
            };
        }
        Err(e) => {
            log::warn!("Jack server is unavailable: {}", e);
        }
    }

    MidiBackendInfo {
        backend: MidiBackend::JackLinux,
        version: None,
        status: MidiBackendStatus::NotRunning,
    }
}
