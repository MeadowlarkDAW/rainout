use std::sync::Once;

use bitflags::bitflags;
use wasapi::{DeviceCollection, Direction};

static INIT: Once = Once::new();

// Defined at https://docs.microsoft.com/en-us/windows/win32/coreaudio/device-state-xxx-constants
bitflags! {
    struct DeviceState: u32 {
        const ACTIVE = 0b00000001;
        const DISABLED = 0b00000010;
        const NOTPRESENT = 0b00000100;
        const UNPLUGGED = 0b00001000;
        const ALL = 0b00001111;
    }
}

use crate::{
    AudioBackendOptions, AudioDeviceOptions, Backend, BackendStatus, BlockSizeRange, ChannelLayout,
    DeviceID,
};

pub(super) fn check_init() {
    // Initialize this only once.
    INIT.call_once(|| initialize_mta().unwrap());
}

pub fn enumerate_audio_backend() -> AudioBackendOptions {
    log::debug!("Enumerating WASAPI server...");

    check_init();

    let coll = match DeviceCollection::new(&Direction::Render) {
        Ok(coll) => coll,
        Err(e) => {
            log::error!("Failed to get WASAPI device collection: {}", e);
            return AudioBackendOptions {
                backend: Backend::Wasapi,
                version: None,
                status: BackendStatus::Error,
                device_options: None,
            };
        }
    };

    let num_devices = match coll.get_nbr_devices() {
        Ok(num_devices) => num_devices,
        Err(e) => {
            log::error!("Failed to get number of WASAPI devices: {}", e);
            return AudioBackendOptions {
                backend: Backend::Wasapi,
                version: None,
                status: BackendStatus::Error,
                device_options: None,
            };
        }
    };

    for i in 0..num_devices {
        match coll.get_device_at_index(i) {
            Ok(device) => {
                match device.get_id() {
                    Ok(device_id) => {
                        let device_name = match device.get_friendlyname() {
                            Ok(name) => name,
                            Err(e) => {
                                log::warn!(
                                    "Failed to get name of WASAPI device with ID {}: {}",
                                    &device_id,
                                    e
                                );
                                String::from("unkown device")
                            }
                        };

                        match device.get_state() {
                            Ok(state) => {
                                match DeviceState::from_bits(bits) {
                                    Some(state) => {
                                        // What a weird API of using bit flags for each of the different
                                        // states the device can be in.
                                        if state.contains(DeviceState::DISABLED) {
                                            log::warn!("The WASAPI device {} has been disabled by the user", &device_name);
                                        } else if state.contains(DeviceState::NOTPRESENT) {
                                            log::warn!(
                                                "The WASAPI device {} is not present",
                                                &device_name
                                            );
                                        } else {
                                            device_options.push(DeviceID {
                                                name: device_name,
                                                identifier: Some(device_id),
                                            })
                                        }
                                    }
                                    None => {
                                        log::error!(
                                            "Got invalid state {} for WASAPI device {}",
                                            state,
                                            &device_name
                                        );
                                    }
                                }
                            }
                            Err(e) => {
                                log::error!(
                                    "Failed to get state of WASAPI device {}: {}",
                                    &name,
                                    e
                                );
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to get ID of WASAPI device at index {}: {}", i, e);
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to get info of WASAPI device at index {}: {}", i, e);
            }
        }
    }

    if device_opts.is_empty() {
        AudioBackendOptions {
            backend: Backend::Wasapi,
            version: None,
            status: BackendStatus::NoDevices,
            device_options: None,
        }
    } else {
        AudioBackendOptions {
            backend: Backend::Wasapi,
            version: None,
            status: BackendStatus::Running,
            device_options: Some(AudioDeviceOptions::SingleDeviceOnly { options: device_opts }),
        }
    }
}

pub fn enumerate_audio_device(device: &DeviceID) -> Result<AudioDeviceConfigOptions, ()> {
    log::debug!("Enumerating WASAPI device {} ...", &device.name);

    check_init();

    let (id, wdevice, jack_unpopulated) = match find_device(device) {
        Ok((id, device, jack_unpopulated)) => (id, device, jack_unpopulated),
        None => return Err(()),
    };

    let audio_client = match wdevice.get_iaudioclient() {
        Ok(audio_client) => audio_client,
        Err(e) => {
            log::error!("Failed to get audio client from WASAPI device {}: {}", &id.name, e);
            return Err(());
        }
    };

    // Get the default format for this device.
    let default_format = match audio_client.get_mixformat() {
        Ok(format) => format,
        Err(e) => {
            log::error!("Failed to get default wave format of WASAPI device {}: {}", &id.name, e);
            return Err(());
        }
    };

    // We only care about channels and sample rate, and not the sample type.
    // We will always convert to/from `f32` buffers  anyway.
    let default_sample_type = match default_format.get_subformat(&self) {
        Ok(s) => s,
        Err(e) => {
            log::error!("Failed to get default wave format of WASAPI device {}: {}", &id.name, e);
            return Err(());
        }
    };
    let default_bps = default_format.bitspersample();
    let default_vbps = default_format.validbitspersample();
    let default_sample_rate = default_format.samplespersec();
    let default_num_channels = default_format.get_nchannels();
    let default_buffer_size = match default_format.get_bufferframecount() {
        Ok(b) => Some(BlockSizeRange { min: b, max: b, default: b }),
        Err(e) => {
            log::debug!("Could not get default buffer size of WASAPI device {}: {}", &id.name, e);
            None
        }
    };

    // TODO: Get channel mask from default format.
    let channel_layout = ChannelLayout::Unspecified;

    // Check if this device supports running in exclusive mode.
    let supports_exclusive = match audio_client.is_supported(
        &wasapi::WaveFormat::new(
            default_bps,
            default_vbps,
            &default_sample_type,
            default_sample_rate,
            default_num_channels,
        ),
        &wasapi::ShareMode::Exclusive,
    ) {
        Ok(None) => true,
        Err(e) => {
            log::error!("Error while enumerating WASAPI device {}: {}", &id.name, e);
            false
        }
        _ => false,
    };

    if supports_exclusive {
        // Search through each common sample rate to see what is supported.
        const sample_rates: [u32; 7] = [22_050, 44_100, 48_000, 88_200, 96_000, 176_400, 192_000];
        let mut supported_sample_rates = Vec::new();
        for sr in sample_rates.iter() {
            match audio_client.is_supported(
                &wasapi::WaveFormat::new(
                    default_bps,
                    default_vbps,
                    &default_sample_type,
                    sr as usize,
                    default_num_channels,
                ),
                &wasapi::ShareMode::Exclusive,
            ) {
                Ok(None) => {
                    supported_sample_rates.push(*sr);
                }
                Err(e) => {
                    log::error!("Error while enumerating WASAPI device {}: {}", &id.name, e);
                }
                _ => (),
            }
        }

        Ok(AudioDeviceConfigOptions {
            sample_rates: Some(supported_sample_rates),
            block_sizes: default_buffer_size,

            num_in_channels: 0,
            num_out_channels: default_num_channels,

            in_channel_layout: ChannelLayout::Unspecified,
            out_channel_layout: channel_layout,

            can_take_exclusive_access: true,

            in_jack_is_unpopulated: false,
            out_jack_is_unpopulated: jack_unpopulated,
        })
    } else {
        // We must use the default config when running in shared mode.

        Ok(AudioDeviceConfigOptions {
            sample_rates: Some(vec![default_sample_rate]),
            block_sizes: default_buffer_size,

            num_in_channels: 0,
            num_out_channels: default_num_channels,

            in_channel_layout: ChannelLayout::Unspecified,
            out_channel_layout: channel_layout,

            can_take_exclusive_access: false,

            in_jack_is_unpopulated: false,
            out_jack_is_unpopulated: jack_unpopulated,
        })
    }
}

pub(super) fn find_device(device: &DeviceID) -> Option<(DeviceID, wasapi::Device, bool)> {
    log::debug!("Finding WASAPI device {} ...", &device.name);

    let coll = match DeviceCollection::new(&Direction::Render) {
        Ok(coll) => coll,
        Err(e) => {
            log::error!("Failed to get WASAPI device collection: {}", e);
            return None;
        }
    };

    let num_devices = match coll.get_nbr_devices() {
        Ok(num_devices) => num_devices,
        Err(e) => {
            log::error!("Failed to get number of WASAPI devices: {}", e);
            return None;
        }
    };

    for i in 0..num_devices {
        match coll.get_device_at_index(i) {
            Ok(d) => {
                match d.get_id() {
                    Ok(device_id) => {
                        let device_name = match device.get_friendlyname() {
                            Ok(name) => name,
                            Err(e) => {
                                log::warn!(
                                    "Failed to get name of WASAPI device with ID {}: {}",
                                    &device_id,
                                    e
                                );
                                String::from("unkown device")
                            }
                        };

                        match device.get_state() {
                            Ok(state) => {
                                match DeviceState::from_bits(bits) {
                                    Some(state) => {
                                        // What a weird API of using bit flags for each of the different
                                        // states the device can be in.
                                        if state.contains(DeviceState::DISABLED) {
                                            log::warn!("The WASAPI device {} has been disabled by the user", &device_name);
                                        } else if state.contains(DeviceState::NOTPRESENT) {
                                            log::warn!(
                                                "The WASAPI device {} is not present",
                                                &device_name
                                            );
                                        } else {
                                            let id = DeviceID {
                                                name: device_name,
                                                identifier: Some(device_id),
                                            };

                                            if id == device {
                                                let jack_unpopulated =
                                                    state.contains(DeviceState::UNPLUGGED);

                                                return Some((id, d, jack_unpopulated));
                                            }
                                        }
                                    }
                                    None => {
                                        log::error!(
                                            "Got invalid state {} for WASAPI device {}",
                                            state,
                                            &device_name
                                        );
                                    }
                                }
                            }
                            Err(e) => {
                                log::error!(
                                    "Failed to get state of WASAPI device {}: {}",
                                    &name,
                                    e
                                );
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to get ID of WASAPI device at index {}: {}", i, e);
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to get info of WASAPI device at index {}: {}", i, e);
            }
        }
    }

    None
}
