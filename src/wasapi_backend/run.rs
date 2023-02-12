use std::error::Error;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc, Arc,
};

use ringbuf::RingBuffer;
use wasapi::SampleType;

const PREALLOC_FRAMES: usize = 48_000;

use crate::{
    error::{ChangeBlockSizeError, RunConfigError, StreamError},
    ProcessInfo,
};
use crate::{
    AudioBufferStreamInfo, AudioDeviceConfig, AudioDeviceStreamInfo, AutoOption, Backend,
    BlockSizeRange, ChannelLayout, DeviceID, PlatformStreamHandle, ProcessHandler, RainoutConfig,
    RainoutDirection, RunOptions, StreamHandle, StreamInfo, StreamMsg,
};

#[cfg(feature = "midi")]
use crate::{error::ChangeMidiPortsError, MidiPortConfig};

pub fn estimated_sample_rate_and_latency(
    config: &RainoutConfig,
) -> Result<(Option<u32>, Option<u32>), RunConfigError> {
    // TODO: Do this properly
    Ok((None, None))
}

pub fn run<P: ProcessHandler>(
    config: &RainoutConfig,
    options: &RunOptions,
    process_handler: P,
) -> Result<StreamHandle<P>, RunConfigError> {
    let (res_tx, res_rx) = mpsc::channel::<Result<StreamHandle<P>, RunConfigError>>();

    let config = config.clone();
    let options = options.clone();

    // TODO: Make sure we spawn a thread with high priority.
    std::thread::spawn(move || match spawn_stream(config, options, process_handler) {
        Ok((stream_handle, audio_thread)) => {
            res_tx.send(Ok(stream_handle)).unwrap();

            audio_thread.run();
        }
        Err(e) => {
            res_tx.send(Err(e)).unwrap();
        }
    });

    // Wait for the returned value.
    match res_rx.recv_timeout(std::time::Duration::from_secs(10)) {
        Ok(res) => res,
        Err(e) => {
            log::error!("Spawning WASAPI thread timed out.");
            Err(RunConfigError::TimedOut)
        }
    }
}

fn spawn_stream<P: ProcessHandler>(
    config: RainoutConfig,
    options: RunOptions,
    mut process_handler: P,
) -> Result<(StreamHandle<P>, AudioThread<P>), RunConfigError> {
    super::check_init();

    let (id, device) = match &config.audio_device {
        AudioDeviceConfig::Auto => match wasapi::get_default_device(&wasapi::Direction::Render) {
            Ok(device) => {
                let name = match device.get_friendlyname() {
                    Ok(n) => n,
                    Err(e) => {
                        log::warn!("Failed to get name of default WASAPI device: {}", e);
                        String::from("unkown device")
                    }
                };

                let identifier = match device.get_id() {
                    Ok(id) => Some(id),
                    Err(e) => {
                        log::warn!("Failed to get ID of WASAPI device {}: {}", &name, e);
                        None
                    }
                };

                (DeviceID { name, identifier }, device)
            }
            Err(e) => {
                return Err(RunConfigError::PlatformSpecific(format!("{}", e)));
            }
        },
        AudioDeviceConfig::Single(device_id) => {
            if let Some((id, device, _jack_unpopulated)) =
                super::find_device(device_id, &config.direction)
            {
                (id, device)
            } else {
                return Err(RunConfigError::AudioDeviceNotFound(device_id.clone()));
            }
        }
        AudioDeviceConfig::LinkedInOut { .. } => {
            return Err(RunConfigError::MalformedConfig(String::from(
                "WASAPI backend does not support linked in/out devices",
            )));
        }
        #[cfg(any(feature = "jack-linux", feature = "jack-macos", feature = "jack-windows"))]
        AudioDeviceConfig::Jack { .. } => {
            return Err(RunConfigError::MalformedConfig(String::from(
                "Jack device in WASAPI backend",
            )));
        }
    };

    let mut audio_client = device.get_iaudioclient()?;
    let default_format = audio_client.get_mixformat()?;
    let default_sample_type = default_format.get_subformat()?;
    let (default_period, min_period) = audio_client.get_periods()?;
    let default_bps = default_format.get_bitspersample();
    let default_vbps = default_format.get_validbitspersample();
    let default_sample_rate = default_format.get_samplespersec();
    let default_num_channels = default_format.get_nchannels();

    // TODO: Get channel mask from default format.
    let channel_layout = ChannelLayout::Unspecified;

    // Check that the device has at-least two output channels.
    if config.direction == RainoutDirection::Render {
        if default_num_channels < 2 && options.must_have_stereo_output {
            return Err(RunConfigError::AutoNoStereoOutputFound);
        }
    }

    // Check if this device supports running in exclusive mode.
    let (share_mode, sample_rate, bps, vbps, sample_type, period) = if config.take_exclusive_access
    {
        let supports_exclusive = match audio_client.is_supported(
            &wasapi::WaveFormat::new(
                default_bps as usize,
                default_vbps as usize,
                &default_sample_type,
                default_sample_rate as usize,
                default_num_channels as usize,
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
        let sample_rate = if let AutoOption::Use(sample_rate) = config.sample_rate {
            let can_use_sample_rate = match audio_client.is_supported(
                &wasapi::WaveFormat::new(
                    default_bps as usize,
                    default_vbps as usize,
                    &default_sample_type,
                    sample_rate as usize,
                    default_num_channels as usize,
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
            sample_rate
        } else {
            default_sample_rate
        };

        // See if this device supports `f32` bit buffers directly.
        let (bps, vbps, sample_type) = match audio_client.is_supported(
            &wasapi::WaveFormat::new(
                32,
                32,
                &wasapi::SampleType::Float,
                sample_rate as usize,
                default_num_channels as usize,
            ),
            &wasapi::ShareMode::Exclusive,
        ) {
            Ok(None) => (32, 32, wasapi::SampleType::Float),
            Ok(Some(format)) => {
                // Use this next-best option given to us.
                match format.get_subformat() {
                    Ok(sample_type) => {
                        let bps = format.get_bitspersample();
                        let vbps = format.get_validbitspersample();

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
                }
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

    let desired_format = wasapi::WaveFormat::new(
        bps as usize,
        vbps as usize,
        &sample_type,
        sample_rate as usize,
        default_num_channels as usize,
    );

    let block_align = desired_format.get_blockalign();

    // TODO: MIDI stuff

    let direction = match config.direction {
        crate::RainoutDirection::Render => wasapi::Direction::Render,
        crate::RainoutDirection::Capture => wasapi::Direction::Capture,
    };
    audio_client.initialize_client(&desired_format, period, &direction, &share_mode, false)?;

    let h_event = audio_client.set_get_eventhandle()?;

    let render_client = match direction {
        wasapi::Direction::Render => Some(audio_client.get_audiorenderclient()?),
        wasapi::Direction::Capture => None,
    };
    let capture_client = match direction {
        wasapi::Direction::Render => None,
        wasapi::Direction::Capture => Some(audio_client.get_audiocaptureclient()?),
    };

    audio_client.start_stream()?;

    let stream_dropped = Arc::new(AtomicBool::new(false));
    let stream_dropped_clone = Arc::clone(&stream_dropped);

    let (to_handle_tx, from_audio_thread_rx) =
        RingBuffer::<StreamMsg>::new(options.msg_buffer_size).split();

    let stream_info = StreamInfo {
        audio_backend: Backend::Wasapi,
        audio_backend_version: None,
        audio_device: AudioDeviceStreamInfo::Single { id, connected_to_system: true },
        sample_rate,
        buffer_size: AudioBufferStreamInfo::UnfixedWithMaxSize(options.max_buffer_size),
        num_in_channels: 0,
        num_out_channels: default_num_channels as u32,
        in_channel_layout: ChannelLayout::Unspecified,
        out_channel_layout: channel_layout,
        estimated_latency: None,           // TODO: Get estimated latency.
        checking_for_silent_inputs: false, // We don't support inputs with WASAPI.

        #[cfg(feature = "midi")]
        midi_info: None, // TODO
    };

    process_handler.init(&stream_info);

    Ok((
        StreamHandle {
            messages: from_audio_thread_rx,
            platform_handle: Box::new(WasapiStreamHandle { stream_info, stream_dropped }),
        },
        AudioThread {
            stream_dropped: stream_dropped_clone,
            audio_client,
            h_event,
            render_client,
            capture_client,
            block_align: block_align as usize,
            vbps,
            sample_type,
            channels: default_num_channels as usize,
            to_handle_tx,
            max_frames: options.max_buffer_size as usize,
            process_handler,
        },
    ))
}

struct AudioThread<P: ProcessHandler> {
    stream_dropped: Arc<AtomicBool>,
    audio_client: wasapi::AudioClient,
    h_event: wasapi::Handle,
    render_client: Option<wasapi::AudioRenderClient>,
    capture_client: Option<wasapi::AudioCaptureClient>,
    block_align: usize,
    vbps: u16,
    sample_type: wasapi::SampleType,
    channels: usize,
    to_handle_tx: ringbuf::Producer<StreamMsg>,
    max_frames: usize,
    process_handler: P,
}

impl<P: ProcessHandler> AudioThread<P> {
    fn run(self) {
        let AudioThread {
            stream_dropped,
            audio_client,
            h_event,
            render_client,
            capture_client,
            block_align,
            vbps,
            sample_type,
            channels,
            mut to_handle_tx,
            max_frames,
            mut process_handler,
        } = self;

        // The buffer that is sent to WASAPI. Pre-allocate a reasonably large size.
        let mut device_buffer = vec![0u8; PREALLOC_FRAMES * block_align];
        let mut device_buffer_capacity_frames = PREALLOC_FRAMES;

        // The owned buffers whose slices get sent to the process method in chunks.
        let mut proc_owned_buffers: Vec<Vec<f32>> =
        (0..channels).map(|_| vec![0.0; max_frames as usize]).collect();
        
        // The buffer that is received from WASAPI. Pre-allocate a reasonably large size.
        let mut device_read_buffer = vec![0u8; PREALLOC_FRAMES * block_align];
        // The owned buffers whose slices get sent to the process method in chunks.
        let mut proc_owned_read_buffers: Vec<Vec<f32>> =
            (0..channels).map(|_| vec![0.0; max_frames as usize]).collect();

        let channel_align = block_align / channels;

        match sample_type {
            wasapi::SampleType::Float => {
                log::info!("WASAPI sample type: SampleType::Float");
            }
            wasapi::SampleType::Int => {
                log::info!("WASAPI sample type: SampleType::Int");
            }
        }
        log::info!("WASAPI stream bits per sample: {}", vbps);

        while !stream_dropped.load(Ordering::Relaxed) {
            let buffer_frame_count = match audio_client.get_available_space_in_frames() {
                Ok(f) => f as usize,
                Err(e) => {
                    log::error!("Fatal WASAPI stream error getting buffer frame count: {}", e);
                    to_handle_tx
                        .push(StreamMsg::Error(StreamError::PlatformSpecific(format!("{}", e))))
                        .unwrap();
                    break;
                }
            };

            // Make sure that the device's buffer is large enough. In theory if we pre-allocated
            // enough frames this shouldn't ever actually trigger any allocation.
            if buffer_frame_count > device_buffer_capacity_frames {
                device_buffer_capacity_frames = buffer_frame_count;
                log::warn!("WASAPI wants a buffer of size {}. This may trigger an allocation on the audio thread.", buffer_frame_count);
                device_buffer.resize(buffer_frame_count as usize * block_align, 0);
            }

            // Read from the device, so we can pass this into the processer the audiothread can read
            if let Some(ref capturer) = capture_client {
                let result = capturer.read_from_device(block_align, &mut device_read_buffer);
                match result {
                    Ok((nbr_frames_returned, buf_flags)) => {
                        // figure out error handling
                        if buf_flags.data_discontinuity {
                            log::error!("Data discontinuity when reading from device");
                        }

                        for b in proc_owned_read_buffers.iter_mut() {
                            b.clear();
                            b.resize(nbr_frames_returned as usize, 0.0);
                        }
                        if !buf_flags.silent {
                            match sample_type {
                                wasapi::SampleType::Float => {
                                    if vbps == 32 {
                                        for j in 0..nbr_frames_returned as usize {
                                            for i in 0..channels {
                                                let offset = (i + j * channels) * 4;
                                                // I feel a bit iffy about this
                                                let val_bytes: [u8; 4] = [
                                                    device_read_buffer[offset],
                                                    device_read_buffer[offset + 1],
                                                    device_read_buffer[offset + 2],
                                                    device_read_buffer[offset + 3],
                                                ];
                                                let val_float = f32::from_le_bytes(val_bytes);
                                                // *val = val_float;
                                                proc_owned_read_buffers[i][j] = val_float;
                                            }
                                        }
                                    }
                                }
                                wasapi::SampleType::Int => {
                                    // TODO: Convert from int to float

                                }
                            }
                            
                        }
                    }
                    Err(e) => {
                        log::error!("Fatal WASAPI stream error while writing to device: {}", e);
                        to_handle_tx
                            .push(StreamMsg::Error(StreamError::PlatformSpecific(format!("{}", e))))
                            .unwrap();
                        break;
                    }
                }
            }

            let mut frames_written = 0;
            while frames_written < buffer_frame_count {
                let frames = (buffer_frame_count - frames_written).min(max_frames);
                
                // Clear and resize the buffer first. Since we never allow more than
                // `max_frames`, this will never allocate.
                for b in proc_owned_buffers.iter_mut() {
                    b.clear();
                    b.resize(frames, 0.0);
                }

                process_handler.process(ProcessInfo {
                    audio_inputs: proc_owned_read_buffers.as_slice(),
                    audio_outputs: proc_owned_buffers.as_mut_slice(),
                    frames,
                    silent_audio_inputs: &[],

                    #[cfg(feature = "midi")]
                    midi_inputs: &[],
                    #[cfg(feature = "midi")]
                    midi_outputs: &mut [],
                });

                let device_buffer_part = &mut device_buffer
                    [frames_written * block_align..(frames_written + frames) * block_align];

                // Fill each slice into the device's output buffer
                //
                // TODO: This could be potentially optimized with unsafe bounds check eliding.
                match sample_type {
                    wasapi::SampleType::Float => {
                        if vbps == 32 {
                            for (frame_i, out_frame) in
                                device_buffer_part.chunks_exact_mut(block_align).enumerate()
                            {
                                for (ch_i, out_smp_bytes) in
                                    out_frame.chunks_exact_mut(channel_align).enumerate()
                                {
                                    let smp_bytes = proc_owned_buffers[ch_i][frame_i].to_le_bytes();

                                    out_smp_bytes[0..smp_bytes.len()].copy_from_slice(&smp_bytes);
                                }
                            }
                        } // TODO: 64 bit buffers?
                    }
                    wasapi::SampleType::Int => {
                        // TODO: Convert from float to int
                    }
                }

                frames_written += frames;
            }

            // Write the now filled output buffer to the device.
            if let Some(ref renderer) = render_client {
                if let Err(e) = renderer.write_to_device(
                    buffer_frame_count as usize,
                    block_align,
                    &device_buffer[0..buffer_frame_count * block_align],
                    None,
                ) {
                    log::error!("Fatal WASAPI stream error while writing to device: {}", e);
                    to_handle_tx
                        .push(StreamMsg::Error(StreamError::PlatformSpecific(format!("{}", e))))
                        .unwrap();
                    break;
                }
            }

            if let Err(e) = h_event.wait_for_event(1000) {
                log::error!("Fatal WASAPI stream error while waiting for event: {}", e);
                to_handle_tx
                    .push(StreamMsg::Error(StreamError::PlatformSpecific(format!("{}", e))))
                    .unwrap();
                break;
            }
        }

        if let Err(e) = audio_client.stop_stream() {
            log::error!("Error stopping WASAPI stream: {}", e);
        }

        log::debug!("WASAPI audio thread ended");
    }
}

pub struct WasapiStreamHandle {
    stream_info: StreamInfo,

    stream_dropped: Arc<AtomicBool>,
}

impl<P: ProcessHandler> PlatformStreamHandle<P> for WasapiStreamHandle {
    fn stream_info(&self) -> &StreamInfo {
        &self.stream_info
    }

    fn change_block_size(&mut self, _block_size: u32) -> Result<(), ChangeBlockSizeError> {
        Err(ChangeBlockSizeError::NotSupportedByBackend)
    }

    #[cfg(feature = "midi")]
    fn change_midi_ports(
        &mut self,
        _in_devices: Vec<MidiPortConfig>,
        _out_devices: Vec<MidiPortConfig>,
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

impl Drop for WasapiStreamHandle {
    fn drop(&mut self) {
        self.stream_dropped.store(true, Ordering::Relaxed);
    }
}

impl From<Box<dyn Error>> for RunConfigError {
    fn from(e: Box<dyn Error>) -> Self {
        RunConfigError::PlatformSpecific(format!("{}", e))
    }
}
