mod stream_handle;
use std::thread;

use ringbuf::RingBuffer;
use stream_handle::WasapiStreamHandle;
use wasapi::initialize_sta;

use crate::{
    error::RunConfigError, AudioBackendOptions, AudioBufferStreamInfo, AudioDeviceConfigOptions,
    AudioDeviceOptions, AudioDeviceStreamInfo, Backend, BackendStatus, ChannelLayout, DeviceID,
    ProcessHandler, ProcessInfo, RainoutConfig, RunOptions, StreamHandle, StreamInfo,
};

fn empty_err<T, U>(res: Result<T, U>) -> Result<T, ()> {
    match res {
        Ok(res_inner) => Ok(res_inner),
        Err(_) => Err(()),
    }
}

fn run_config_err<T>(res: Result<T, Box<dyn std::error::Error>>) -> Result<T, RunConfigError> {
    match res {
        Ok(inner) => Ok(inner),
        Err(err) => Err(RunConfigError::PlatformSpecific(err)),
    }
}

fn get_device_ids(collection: &wasapi::DeviceCollection) -> Result<Vec<DeviceID>, ()> {
    let mut result = Vec::new();

    for i in 0..empty_err(collection.get_nbr_devices())? {
        let device = empty_err(collection.get_device_at_index(i))?;

        result.push(DeviceID {
            name: empty_err(device.get_friendlyname())?,
            identifier: Some(empty_err(device.get_id())?),
        });
    }

    Ok(result)
}

pub fn enumerate_audio_backend() -> Result<AudioBackendOptions, ()> {
    // TODO: No idea if this is cheap
    empty_err(initialize_sta())?;

    let input_devices = empty_err(wasapi::DeviceCollection::new(&wasapi::Direction::Capture))?;
    let output_devices = empty_err(wasapi::DeviceCollection::new(&wasapi::Direction::Render))?;

    let device_count =
        empty_err(input_devices.get_nbr_devices())? + empty_err(output_devices.get_nbr_devices())?;

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

pub fn enumerate_audio_device(_device: &DeviceID) -> Result<AudioDeviceConfigOptions, ()> {
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

fn get_default_device(direction: &wasapi::Direction) -> Result<DeviceID, ()> {
    empty_err(initialize_sta())?;

    let device = empty_err(wasapi::get_default_device(direction))?;

    Ok(DeviceID {
        name: empty_err(device.get_friendlyname())?,
        identifier: Some(empty_err(device.get_id())?),
    })
}

fn id_to_device(
    id: &DeviceID,
    device_collection: &wasapi::DeviceCollection,
) -> Result<wasapi::Device, ()> {
    empty_err(initialize_sta())?;

    Ok(empty_err(device_collection.get_device_with_name(id.name.as_str()))?)
}

struct DeviceRenderInfo {
    pub audio_client: wasapi::AudioClient,
    pub render_client: wasapi::AudioRenderClient,
    pub blockalign: usize,
    pub client_buffer: [Vec<f32>; 2],
    pub wasapi_buffer: Vec<u8>,
    pub h_event: wasapi::Handle,
}

pub fn run<P: ProcessHandler>(
    config: &RainoutConfig,
    options: &RunOptions,
    mut process_handler: P,
) -> Result<StreamHandle<P>, RunConfigError> {
    if let Err(err) = initialize_sta() {
        return Err(RunConfigError::PlatformSpecific(Box::new(err)));
    };

    println!("{:#?}", config);
    println!("{:#?}", options);

    let audio_device_id = match &config.audio_device {
        crate::AudioDeviceConfig::Single(device) => {
            AudioDeviceStreamInfo::Single { id: device.clone(), connected_to_system: false }
        }
        crate::AudioDeviceConfig::LinkedInOut { input, output } => {
            AudioDeviceStreamInfo::LinkedInOut {
                input: input.clone(),
                output: output.clone(),
                in_connected_to_system: false,
                out_connected_to_system: false,
            }
        }
        crate::AudioDeviceConfig::Jack { in_ports: _, out_ports: _ } => {
            return Err(RunConfigError::MalformedConfig(
                "WASAPI does not support JACK devices".to_string(),
            ));
        }
        crate::AudioDeviceConfig::Auto => AudioDeviceStreamInfo::LinkedInOut {
            input: get_default_device(&wasapi::Direction::Capture).ok(),
            output: get_default_device(&wasapi::Direction::Render).ok(),
            in_connected_to_system: false, // Not sure what these mean so ignoring them for now
            out_connected_to_system: false,
        },
    };

    let stream_info = StreamInfo {
        audio_backend: Backend::Wasapi,
        audio_backend_version: None,
        audio_device: audio_device_id.clone(),
        sample_rate: match config.sample_rate {
            crate::AutoOption::Use(sample_rate) => sample_rate,
            crate::AutoOption::Auto => 44100,
        },
        buffer_size: match config.block_size {
            crate::AutoOption::Use(block_size) => {
                // This is possible, but I'm not sure how
                // AudioBufferStreamInfo::UnfixedWithMinSize(block_size)
                //
                // We'll just silently use unfixed for now
                AudioBufferStreamInfo::Unfixed
            }
            crate::AutoOption::Auto => AudioBufferStreamInfo::Unfixed,
        },
        estimated_latency: None,           // TODO: no idea
        checking_for_silent_inputs: false, // TODO: ??
        midi_info: None,
    };

    process_handler.init(&stream_info);

    let (to_stream_handle_tx, from_audio_thread_rx) =
        RingBuffer::new(options.msg_buffer_size).split();

    // Playback thread
    thread::spawn(move || {
        // TODO: Better error handling
        initialize_sta().unwrap();

        let out_device_collection =
            run_config_err(wasapi::DeviceCollection::new(&wasapi::Direction::Render)).unwrap();
        let in_device_collection =
            run_config_err(wasapi::DeviceCollection::new(&wasapi::Direction::Capture)).unwrap();

        let out_device = match &audio_device_id {
            AudioDeviceStreamInfo::Single { id, connected_to_system } => {
                match id_to_device(id, &out_device_collection) {
                    Ok(device) => Some(device),
                    // Err(_) => return Err(RunConfigError::AudioDeviceNotFound(id.name.clone())),
                    Err(_) => todo!(),
                }
            }
            AudioDeviceStreamInfo::LinkedInOut {
                input,
                output,
                in_connected_to_system,
                out_connected_to_system,
            } => match output {
                Some(id) => match id_to_device(id, &out_device_collection) {
                    Ok(device) => Some(device),
                    // Err(_) => return Err(RunConfigError::AudioDeviceNotFound(id.name.clone())),
                    Err(_) => todo!(),
                },
                None => None,
            },
            AudioDeviceStreamInfo::Jack { in_ports: _, out_ports: _ } => unreachable!(),
        };

        let mut out_render_info = if let Some(device) = out_device {
            // TODO: Get rid of unwraps here

            let mut audio_client = run_config_err(device.get_iaudioclient()).unwrap();

            let desired_format = wasapi::WaveFormat::new(
                32,
                32,
                &wasapi::SampleType::Float,
                stream_info.sample_rate as usize,
                2,
            );

            let blockalign = desired_format.get_blockalign() as usize;

            let (def_time, min_time) = run_config_err(audio_client.get_periods()).unwrap();

            run_config_err(audio_client.initialize_client(
                &desired_format,
                min_time as i64,
                &wasapi::Direction::Render,
                &wasapi::ShareMode::Shared,
                true,
            ))
            .unwrap();

            let h_event = run_config_err(audio_client.set_get_eventhandle()).unwrap();

            let mut buffer_frame_count =
                run_config_err(audio_client.get_bufferframecount()).unwrap();

            let render_client = run_config_err(audio_client.get_audiorenderclient()).unwrap();

            let frames = 512;

            let mut out_buffer = [vec![0f32; frames], vec![0f32; frames]];

            let mut client_buffer = [vec![0f32], vec![0f32]];
            let mut wasapi_buffer = Vec::new();

            Some(DeviceRenderInfo {
                audio_client,
                render_client,
                blockalign,
                client_buffer,
                wasapi_buffer,
                h_event,
            })
        } else {
            None
        };

        if out_render_info.is_some() {
            out_render_info.as_ref().unwrap().audio_client.start_stream().unwrap();
        }

        if out_render_info.is_none() {
            return;
        }

        loop {
            if let Some(render_data) = &mut out_render_info {
                let frame_count =
                    render_data.audio_client.get_available_space_in_frames().unwrap() as usize; // TODO: Better error handling

                // TODO: We really shouldn't be allocating on the audio thread
                while render_data.client_buffer[0].len() < frame_count {
                    render_data.client_buffer[0].push(0f32);
                    render_data.client_buffer[1].push(0f32);
                }

                // Get samples from the library consumer
                process_handler.process(ProcessInfo {
                    audio_inputs: &[],
                    audio_outputs: &mut render_data.client_buffer,
                    frames: frame_count,
                    silent_audio_inputs: &[],
                    midi_inputs: &[],
                    midi_outputs: &mut [],
                });

                // println!("frame_count: {}", frame_count);
                // println!("channel_count: {}", channel_count);
                // println!("render_data.blockalign: {}", render_data.blockalign);

                // println!("{}", frame_count * channel_count * render_data.blockalign);

                // println!("{}", render_data.wasapi_buffer.len());

                // TODO: Can we avoid allocating here?
                while render_data.wasapi_buffer.len() < frame_count * render_data.blockalign {
                    render_data.wasapi_buffer.push(0u8);
                }

                while render_data.wasapi_buffer.len() > frame_count * render_data.blockalign {
                    render_data.wasapi_buffer.pop();
                }

                assert!(
                    render_data.blockalign
                        == render_data.client_buffer[0][0].to_le_bytes().len()
                            * render_data.client_buffer.len()
                ); // TODO: Remove when we're more sure about this

                // Move the collected samples to the WASAPI buffer
                for i in 0..frame_count {
                    for j in 0..render_data.client_buffer.len() {
                        let client_channel = &render_data.client_buffer[j];

                        let sample_bytes = client_channel[i].to_le_bytes();
                        for k in 0..sample_bytes.len() {
                            (&mut render_data.wasapi_buffer)[i * j * k] = sample_bytes[k];
                        }
                    }
                }

                render_data
                    .render_client
                    .write_to_device(
                        frame_count,
                        render_data.blockalign,
                        &render_data.wasapi_buffer,
                        None,
                    )
                    .unwrap();
                
                if render_data.h_event.wait_for_event(1000).is_err() {
                    render_data.audio_client.stop_stream().unwrap();
                    break;
                    // error!("error, stopping playback"); // TODO: Send an event here, or something
                }
            }
        }
    });

    let stream_handle: StreamHandle<P> = StreamHandle {
        messages: from_audio_thread_rx,
        platform_handle: Box::new(WasapiStreamHandle { stream_info }),
    };

    Ok(stream_handle)
}
