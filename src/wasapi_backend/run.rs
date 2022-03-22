use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use ringbuf::Producer;
use wasapi::WasapiRes;

const PREALLOC_FRAMES: usize = 48_000;

use crate::error::{ChangeBlockSizeError, RunConfigError, StreamError};
use crate::{
    AudioBufferStreamInfo, AudioDeviceConfig, AudioDeviceStreamInfo, AutoOption, Backend, DeviceID,
    PlatformStreamHandle, ProcessHandler, RainoutConfig, RunOptions, StreamHandle, StreamInfo,
    StreamMsg,
};

#[cfg(feature = "midi")]
use crate::{
    error::ChangeMidiPortsError, MidiControlScheme, MidiPortConfig, MidiPortStreamInfo,
    MidiStreamInfo,
};

pub fn estimated_sample_rate_and_latency(
    config: &RainoutConfig,
) -> Result<(Option<u32>, Option<u32>), RunConfigError> {
    // TODO: Do this properly
    Some((None, None))
}

pub fn run<P: ProcessHandler>(
    config: &RainoutConfig,
    options: &RunOptions,
    mut process_handler: P,
) -> Result<StreamHandle<P>, RunConfigError> {
    super::check_init();

    let (id, device) = match &config.audio_device {
        AudioDeviceConfig::Auto => match wasapi::get_default_device(&wasapi::Direction::Render) {
            Ok(device) => {
                let name = match device.get_friendlyname() {
                    Ok(n) => n,
                    Err(e) => {
                        log::warn!("Failed to get name of WASAPI device: {}");
                        String::from("unkown device")
                    }
                };

                let identifier = match device.get_id() {
                    Ok(id) => Some(id),
                    Err(e) => {
                        log::warn!("Failed to get ID of WASAPI device {}: {}", &name, id);
                        None
                    }
                };

                (DeviceID { name, identifier }, device)
            }
            Err(e) => {
                return Err(RunConfigError::PlatformSpecific(e));
            }
        },
        AudioDeviceConfig::Single(device_id) => {
            if let Ok((id, device, _jack_unpopulated)) = super::find_device(device_id) {
                (id, device)
            } else {
                return Err(RunConfigError::AudioDeviceNotFound(device_id.clone()));
            }
        }
        AudioDeviceConfig::LinkedInOut { .. } => {
            return Err(RunConfigError::MalformedConfig(
                "WASAPI backend does not support linked in/out devices",
            ));
        }
        #[cfg(any(feature = "jack-linux", feature = "jack-macos", feature = "jack-windows"))]
        AudioDeviceConfig::Jack { .. } => {
            return Err(RunConfigError::MalformedConfig("Jack device in WASAPI backend"));
        }
    };

    let mut audio_client = device.get_iaudioclient()?;
    let default_sample_type = default_format.get_subformat(&self)?;
    let (default_period, min_period) = default_format.get_periods()?;
    let default_bps = default_format.bitspersample();
    let default_vbps = default_format.validbitspersample();
    let default_blockalign = default_format.getblockalign();
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

    // Check that the device has at-least two output channels.
    if default_num_channels < 2 && options.must_have_stereo_output {
        return Err(RunConfigError::AutoNoStereoOutputFound);
    }

    // Check if this device supports running in exclusive mode.
    let (share_mode, sample_rate, bps, vbps, sample_type, blockalign, period) =
        if config.take_exclusive_access {
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
            if !supports_exclusive {
                return Err(RunConfigError::CouldNotUseExclusive);
            }

            // Check that the device supports the requested sample rate.
            if let AutoOption::Use(sample_rate) = config.sample_rate {
                let can_use_sample_rate = match audio_client.is_supported(
                    &wasapi::WaveFormat::new(
                        default_bps,
                        default_vbps,
                        &default_sample_type,
                        sample_rate,
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
                if !can_use_sample_rate {
                    return Err(RunConfigError::CouldNotUseSampleRate(sample_rate));
                }
            }

            // See if this device supports `f32` bit buffers directly.
            let (bps, vbps, sample_type) = match audio_client.is_supported(
                &wasapi::WaveFormat::new(
                    32,
                    32,
                    &wasapi::SampleType::Float,
                    sample_rate,
                    default_num_channels,
                ),
                &wasapi::ShareMode::Exclusive,
            ) {
                Ok(None) => (32, 32, wasapi::SampleType::Float),
                Ok(format) => {
                    // Use this next-best option given to us.
                    match format.get_subformat(&self) {
                        Ok(sample_type) => {
                            let bps = format.bitspersample();
                            let vbps = format.validbitspersample();

                            (bps, vbps, sample_type)
                        }
                        Err(e) => {
                            log::error!(
                                "Failed to get default wave format of WASAPI device {}: {}",
                                &id.name,
                                e
                            );
                            (default_bps, default_vbps, default_sample_type)
                        }
                    };
                }
                Err(e) => {
                    log::error!("Error while enumerating WASAPI device {}: {}", &id.name, e);
                    (default_bps, default_vbps, default_sample_type)
                }
            };

            (wasapi::ShareMode::Exclusive, sample_rate, bps, vbps, sample_type, min_period)
        } else {
            (
                wasapi::ShareMode::Shared,
                default_sample_rate,
                default_bps,
                default_vbps,
                default_sample_type,
                default_period,
            )
        };

    let desired_format =
        wasapi::WaveFormat::new(bps, vbps, &sample_type, sample_rate, default_num_channels);

    let block_align = desired_format.get_blockalign();

    // TODO: MIDI stuff

    audio_client.initialize_client(
        &desired_format,
        period,
        &wasapi::Direction::Render,
        &share_mode,
        false,
    )?;

    let h_event = audio_client.set_get_eventhandle()?;

    let render_client = audio_client.get_audiorenderclient()?;

    audio_client.start_stream()?;

    let stream_dropped = Arc::new(AtomicBool::new(false));
    let stream_dropped_clone = Arc::clone(&stream_dropped);

    let (to_handle_tx, from_audio_thread_rx) =
        ringbuf::RingBuffer::<StreamMsg>::new(options.msg_buffer_size).split();

    let stream_info = StreamInfo {
        audio_backend: Backend::Wasapi,
        audio_backend_version: None,
        audio_device: AudioDeviceStreamInfo::Single { id, connected_to_system: true },
        sample_rate,
        buffer_size: AudioBufferStreamInfo::UnfixedWithMaxSize(options.max_buffer_size),
        num_in_channels: 0,
        num_out_channels: default_num_channels,
        in_channel_layout: ChannelLayout::Unspecified,
        out_channel_layout: ChannelLayout::Unspecified, // TODO: Get channel layout.
        estimated_latency: None,                        // TODO: Get estimated latency.
        checking_for_silent_inputs: false,              // We don't support inputs with WASAPI.

        #[cfg(feature = "midi")]
        midi_info: None, // TODO
    };

    process_handler.init(&stream_info);

    // TODO: Make sure we spawn a thread with high priority.
    std::thread::spawn(move || {
        audio_thread(
            stream_dropped_clone,
            audio_client,
            h_event,
            render_client,
            block_align as usize,
            default_num_channels,
            to_handle_tx,
            options.max_buffer_size,
            process_handler,
        )
    });

    Ok(StreamHandle {
        messages: from_audio_thread_rx,
        platform_handle: Box::new(WasapiStreamHandle { stream_info, stream_dropped }),
    })
}

fn audio_thread<P: ProcessHandler>(
    stream_dropped: Arc<AtomicBool>,
    audio_client: wasapi::AudioClient,
    h_event: wasapi::Handle,
    render_client: wasapi::AudioRenderClient,
    block_align: usize,
    channels: usize,
    mut to_handle_tx: ringbuf::Producer<StreamMsg>,
    max_frames: usize,
    mut process_handler: P,
) {
    // The buffer that is sent to WASAPI. Pre-allocate a reasonably large size.
    let mut device_buffer = vec![0u8; PREALLOC_FRAMES * blockalign];
    let mut device_buffer_capacity_frames = PREALLOC_FRAMES;

    // The owned buffers whose slices get sent to the process method in chunks.
    let mut proc_owned_buffers = (0..channels).map(|_| vec![0.0; max_frames]).collect();

    while !stream_dropped.load(Ordering::Relaxed) {
        let buffer_frame_count = match audio_client.get_available_space_in_frames() {
            Ok(f) => f,
            Err(e) => {
                log::error!("Fatal WASAPI stream error getting buffer frame count: {}", e);
                to_handle_tx.push(StreamMsg::Error(StreamError::PlatformSpecific(e))).unwrap();
                break;
            }
        };

        // Make sure that the device's buffer is large enough. In theory if we pre-allocated
        // enough frames this shouldn't ever actually trigger any allocation.
        if buffer_frame_count > device_buffer_capacity_frames {
            device_buffer_capacity_frames = buffer_frame_count;
            log::warn!("WASAPI wants a buffer of size {}. This may trigger an allocation on the audio thread.", buffer_frame_count);
            device_buffer.resize((buffer_frame_count * blockalign), 0);
        }

        let mut frames_written = 0;
        while frames_written < buffer_frame_count {
            let frames = frames_left.min(max_frames);

            // Store a slice from each output channel into the process info.
            for (owned_buffer, proc_buffer) in
                proc_owned_buffers.channels.iter_mut().zip(proc_info.audio_outputs.iter_mut())
            {
                let s = &mut owned_buffer[frames_written..frames_written + frames];

                // Clear the buffer first
                s.fill(0.0);

                *proc_buffer = s;
            }

            process_handler.process(proc_info);

            let device_buffer_part = &mut device_buffer
                [frames_written * blockalign..(frames_written + frames) * block_align];

            // Fill each slice into the device's output buffer
            for (frame_i, out_frame) in device_buffer_part.chunks_exact_mut(blockalign).enumerate()
            {
                for (ch_i, out_smp_bytes) in
                    frame.chunks_exact_mut(blockalign / channels).enumerate()
                {
                    let smp_bytes = proc_info.audio_outputs[ch_i][frame_i].to_le_bytes();

                    out_smp_bytes[0..smp_bytes.len()].copy_from_slice(smp_bytes);
                }
            }

            frames_written += frames;
        }

        // Write the now filled output buffer to the device.
        if let Err(e) = render_client.write_to_device(
            buffer_frame_count as usize,
            block_align,
            &device_buffer[0..buffer_frame_count * block_align],
            None,
        ) {
            log::error!("Fatal WASAPI stream error while writing to device: {}", e);
            to_handle_tx.push(StreamMsg::Error(StreamError::PlatformSpecific(e))).unwrap();
            break;
        }

        if let Err(e) = h_event.wait_for_event(1000) {
            log::error!("Fatal WASAPI stream error while waiting for event: {}", e);
            to_handle_tx.push(StreamMsg::Error(StreamError::PlatformSpecific(e))).unwrap();
            break;
        }
    }

    if let Err(e) = audio_client.stop_stream() {
        log::error!("Error stopping WASAPI stream: {}", e);
    }

    log::debug!("WASAPI audio thread ended");
}

pub struct WasapiStreamHandle<P: ProcessHandler> {
    stream_info: StreamInfo,

    stream_dropped: Arc<AtomicBool>,
}

impl<P: ProcessHandler> PlatformStreamHandle<P> for WasapiStreamHandle<P> {
    fn stream_info(&self) -> &StreamInfo {
        &self.stream_info
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
        Err(ChangeMidiPortsError::NotSupportedByBackend)
    }

    fn can_change_block_size(&self) -> bool {
        false
    }

    #[cfg(feature = "midi")]
    fn can_change_midi_ports(&self) -> bool {
        false
    }
}

impl<P: ProcessHandler> Drop for WasapiStreamHandle<P> {
    fn drop(&mut self) {
        self.stream_dropped.store(true, Ordering::Relaxed);
    }
}

impl From<wasapi::WasapiError> for RunConfigError {
    fn from(e: wasapi::WasapiError) -> Self {
        RunConfigError::PlatformSpecific(Box::new(e))
    }
}
