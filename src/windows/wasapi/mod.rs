use std::error::Error;

use wasapi::{initialize_sta, DeviceCollection, Direction};

use crate::{
    AudioBackendOptions, AudioDeviceConfigOptions, AudioDeviceOptions, ChannelLayout, DeviceID,
};

thread_local!(static COM_INITIALIZED: bool = {
    match initialize_sta() {
        Ok(_) => {true}
        Err(_) => {false}
    }
});

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

pub fn enumerate() -> Result<AudioBackendOptions, ()> {
    // TODO: No idea if this is cheap
    initialize_sta();
    let input_devices = convert_result(DeviceCollection::new(&Direction::Capture))?;
    let output_devices = convert_result(DeviceCollection::new(&Direction::Render))?;

    Ok(AudioBackendOptions {
        name: "wasapi",
        version: None,
        device_options: AudioDeviceOptions::LinkedInOutDevice {
            input_devices: get_device_ids(&input_devices)?,
            output_devices: get_device_ids(&output_devices)?,
            config_options: AudioDeviceConfigOptions {
                // WASAPI supports more, but figuring out which is trial-and-
                // error. For now we'll just advertise 44.1k, but we should
                // support more in the future. See CPAL's WASAPI implementation
                // for more.
                sample_rates: Some(vec![44100]),

                // WASAPI gives some control over this, but doesn't guarantee a
                // fixed-size buffer. See here for more:
                // https://stackoverflow.com/q/20371033/8166701
                block_sizes: None,

                num_input_ports: 1,  // ???
                num_output_ports: 1, // ???
                input_channel_layout: ChannelLayout::Stereo,
                output_channel_layout: ChannelLayout::Stereo,

                // TODO: WASAPI has an "exclusive" mode. Verify that this means
                // the same as this.
                can_take_exclusive_access: true,
            },
        },
    })
}
