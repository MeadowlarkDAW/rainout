use std::error::Error;

use wasapi::{initialize_sta, DeviceCollection, Direction};

use crate::{
    error::RunConfigError, AudioBackendOptions, AudioDeviceConfigOptions, AudioDeviceOptions,
    BackendStatus, ChannelLayout, DeviceID, ProcessHandler, RainoutConfig, RunOptions,
    StreamHandle, Backend,
};

// From the wasapi crate
type WasapiRes<T> = Result<T, Box<dyn Error>>;

fn convert_result<T>(res: WasapiRes<T>) -> Result<T, ()> {
    match res {
        Ok(res_inner) => Ok(res_inner),
        Err(_) => Err(()),
    }
}

fn get_device_ids(collection: &DeviceCollection) -> Result<Vec<DeviceID>, ()> {
    let mut result = Vec::new();

    for i in 0..convert_result(collection.get_nbr_devices())? {
        let device = convert_result(collection.get_device_at_index(i))?;

        result.push(DeviceID {
            name: convert_result(device.get_friendlyname())?,
            identifier: Some(convert_result(device.get_id())?),
        });
    }

    Ok(result)
}

pub fn enumerate_audio_backend() -> Result<AudioBackendOptions, ()> {
    // TODO: No idea if this is cheap
    initialize_sta();
    let input_devices = convert_result(DeviceCollection::new(&Direction::Capture))?;
    let output_devices = convert_result(DeviceCollection::new(&Direction::Render))?;

    let device_count = convert_result(input_devices.get_nbr_devices())?
        + convert_result(output_devices.get_nbr_devices())?;

    Ok(AudioBackendOptions {
        backend: Backend::Wasapi,
        version: None,
        device_options: Some(AudioDeviceOptions::LinkedInOutDevice {
            in_devices: get_device_ids(&input_devices)?,
            out_devices: get_device_ids(&output_devices)?,
        }),
        status: match device_count {
            0 => BackendStatus::NoDevices,
            _ => BackendStatus::Running,
        },
    })
}

pub fn enumerate_audio_device(
    backend: &str,
    device: &DeviceID,
) -> Result<AudioDeviceConfigOptions, ()> {
    Ok(AudioDeviceConfigOptions {
        // WASAPI supports more, but figuring out which is trial-and-
        // error. For now we'll just advertise 44.1k, but we should
        // support more in the future. See CPAL's WASAPI implementation
        // for more.
        sample_rates: Some(vec![44100]),

        // WASAPI gives some control over this, but doesn't guarantee a
        // fixed-size buffer. See here for more:
        // https://stackoverflow.com/q/20371033/8166701
        block_sizes: None,

        num_in_channels: 2,  // ???
        num_out_channels: 2, // ???

        // TODO: This probably isn't always true. See:
        // https://stackoverflow.com/q/33047471/8166701
        in_channel_layout: ChannelLayout::Stereo,
        out_channel_layout: ChannelLayout::Stereo,

        // TODO: WASAPI has an "exclusive" mode. Verify that this means
        // the same.
        can_take_exclusive_access: true,
    })
}

pub fn run<P: ProcessHandler>(
    config: &RainoutConfig,
    options: &RunOptions,
    process_handler: P,
) -> Result<StreamHandle<P>, RunConfigError> {
    todo!()
}
