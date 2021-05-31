use std::ffi::CString;

use log::{debug, info, warn};

use alsa::device_name::HintIter;

use crate::{
    AudioDeviceBuffer, AudioDeviceConfig, AudioServerConfig, AudioServerInfo, BufferSizeInfo,
    DeviceIndex, DuplexDeviceInfo, InternalAudioDevice, InternalMidiDevice, MidiDeviceBuffer,
    MidiDeviceConfig, MidiServerInfo, ProcessInfo, RtProcessHandler, SpawnRtThreadError,
    StreamError, StreamInfo, SystemAudioDeviceInfo, SystemMidiDeviceInfo,
};

fn extract_device_pretty_name(desc: &String) -> String {
    let mut i = 0;
    for c in desc.chars() {
        if c == '\n' {
            break;
        }
        i += 1;
    }

    String::from(&desc[0..i])
}

pub fn refresh_audio_server(server: &mut AudioServerInfo) {
    info!("Refreshing list of available ALSA audio devices...");

    server.devices.clear();

    match HintIter::new(None, &*CString::new("pcm").unwrap()) {
        Ok(i) => {
            server.available = true;

            for device_hint in i {
                let device_name = match &device_hint.name {
                    None => continue,
                    Some(n) => {
                        if n == "null" {
                            continue;
                        }
                        n
                    }
                };

                let device_pretty_name = match &device_hint.desc {
                    None => continue,
                    Some(n) => extract_device_pretty_name(n),
                };

                let mut do_get_params = true;
                for device in server.devices.iter() {
                    if &device.name == &device_pretty_name {
                        do_get_params = false;
                        break;
                    }
                }

                let mut in_device = None;
                let mut out_device = None;

                let mut sample_rates = Vec::new();

                let mut min_buffer_size = 0;
                let mut max_buffer_size = 0;

                // Try to open device as input
                if let Ok(pcm) = alsa::pcm::PCM::new(device_name, alsa::Direction::Capture, true) {
                    match alsa::pcm::HwParams::any(&pcm) {
                        Ok(hw_params) => {
                            if do_get_params {
                                // Get supported sample rates

                                let min_rate = match hw_params.get_rate_min() {
                                    Ok(r) => r,
                                    Err(e) => {
                                        debug!(
                                            "Could not get min sample rate of ALSA device {}: {}",
                                            device_name, e
                                        );
                                        continue;
                                    }
                                };
                                let max_rate = match hw_params.get_rate_max() {
                                    Ok(r) => r,
                                    Err(e) => {
                                        debug!(
                                            "Could not get max sample rate of ALSA device {}: {}",
                                            device_name, e
                                        );
                                        continue;
                                    }
                                };

                                sample_rates = if min_rate == max_rate {
                                    vec![min_rate]
                                } else {
                                    const RATES: [u32; 13] = [
                                        5512, 8000, 11025, 16000, 22050, 32000, 44100, 48000,
                                        64000, 88200, 96000, 176400, 192000,
                                    ];

                                    let mut rates = Vec::new();
                                    for &rate in RATES.iter() {
                                        if hw_params.test_rate(rate).is_ok() {
                                            rates.push(rate);
                                        }
                                    }

                                    if rates.is_empty() {
                                        debug!(
                                            "Could not find a working sample rate for ALSA device {}",
                                            device_name
                                        );
                                        continue;
                                    } else {
                                        rates
                                    }
                                };

                                // Get supported buffer sizes

                                min_buffer_size = match hw_params.get_buffer_size_min() {
                                    Ok(b) => b,
                                    Err(e) => {
                                        debug!(
                                            "Could not get min buffer size of ALSA device {}: {}",
                                            device_name, e
                                        );
                                        continue;
                                    }
                                };
                                max_buffer_size = match hw_params.get_buffer_size_max() {
                                    Ok(b) => b,
                                    Err(e) => {
                                        debug!(
                                            "Could not get max buffer size of ALSA device {}: {}",
                                            device_name, e
                                        );
                                        continue;
                                    }
                                };
                            }

                            // Get number of channels

                            let channels = match hw_params.get_channels() {
                                Ok(ch) => ch,
                                Err(e) => {
                                    debug!("Could not get number of input channels of ALSA device {}: {}", device_name, e);
                                    continue;
                                }
                            };

                            in_device = Some(SystemAudioDeviceInfo {
                                name: device_name.clone(),
                                channels: channels as u16,
                            });
                        }
                        Err(e) => {
                            debug!("Could not get params of ALSA device {}: {}", device_name, e)
                        }
                    }
                }

                // Try to open device as output
                if let Ok(pcm) = alsa::pcm::PCM::new(device_name, alsa::Direction::Playback, true) {
                    match alsa::pcm::HwParams::any(&pcm) {
                        Ok(hw_params) => {
                            if do_get_params {
                                // Get supported sample rates

                                let min_rate = match hw_params.get_rate_min() {
                                    Ok(r) => r,
                                    Err(e) => {
                                        debug!(
                                            "Could not get min sample rate of ALSA device {}: {}",
                                            device_name, e
                                        );
                                        continue;
                                    }
                                };
                                let max_rate = match hw_params.get_rate_max() {
                                    Ok(r) => r,
                                    Err(e) => {
                                        debug!(
                                            "Could not get max sample rate of ALSA device {}: {}",
                                            device_name, e
                                        );
                                        continue;
                                    }
                                };

                                sample_rates = if min_rate == max_rate {
                                    vec![min_rate]
                                } else {
                                    const RATES: [u32; 13] = [
                                        5512, 8000, 11025, 16000, 22050, 32000, 44100, 48000,
                                        64000, 88200, 96000, 176400, 192000,
                                    ];

                                    let mut rates = Vec::new();
                                    for &rate in RATES.iter() {
                                        if hw_params.test_rate(rate).is_ok() {
                                            rates.push(rate);
                                        }
                                    }

                                    if rates.is_empty() {
                                        debug!("Could not find a working sample rate for ALSA device {}", device_name);
                                        continue;
                                    } else {
                                        rates
                                    }
                                };

                                // Get supported buffer sizes

                                min_buffer_size = match hw_params.get_buffer_size_min() {
                                    Ok(b) => b,
                                    Err(e) => {
                                        debug!(
                                            "Could not get min buffer size of ALSA device {}: {}",
                                            device_name, e
                                        );
                                        continue;
                                    }
                                };
                                max_buffer_size = match hw_params.get_buffer_size_max() {
                                    Ok(b) => b,
                                    Err(e) => {
                                        debug!(
                                            "Could not get max buffer size of ALSA device {}: {}",
                                            device_name, e
                                        );
                                        continue;
                                    }
                                };
                            }

                            // Get number of channels

                            let channels = match hw_params.get_channels() {
                                Ok(ch) => ch,
                                Err(e) => {
                                    debug!("Could not get number of output channels of ALSA device {}: {}", device_name, e);
                                    continue;
                                }
                            };

                            out_device = Some(SystemAudioDeviceInfo {
                                name: device_name.clone(),
                                channels: channels as u16,
                            });
                        }
                        Err(e) => {
                            debug!("Could not get params of ALSA device {}: {}", device_name, e)
                        }
                    }
                }

                if in_device.is_none() && out_device.is_none() {
                    debug!("No available inputs/outpus in device {}", device_name);
                    continue;
                } else {
                    let duplex_device = {
                        let mut d = None;
                        for device in server.devices.iter_mut() {
                            if &device.name == &device_pretty_name {
                                d = Some(device);
                                break;
                            }
                        }

                        match d {
                            Some(d) => d,
                            None => {
                                server.devices.push(DuplexDeviceInfo {
                                    name: device_pretty_name.clone(),
                                    in_devices: Vec::new(),
                                    out_devices: Vec::new(),
                                    sample_rates,
                                    buffer_size: BufferSizeInfo::Range {
                                        min: min_buffer_size as u32,
                                        max: max_buffer_size as u32,
                                    },
                                });

                                server.devices.last_mut().unwrap()
                            }
                        }
                    };

                    if let Some(in_device) = in_device {
                        duplex_device.in_devices.push(in_device);
                    }
                    if let Some(out_device) = out_device {
                        duplex_device.out_devices.push(out_device);
                    }
                }
            }
        }
        Err(e) => {
            server.available = false;

            info!("ALSA server is unavailable: {}", e);
        }
    }
}

pub fn refresh_midi_server(server: &mut MidiServerInfo) {
    info!("Refreshing list of available ALSA MIDI devices...");

    server.in_devices.clear();
    server.out_devices.clear();

    let mut in_seq_available = false;
    match alsa::Seq::open(None, Some(alsa::Direction::Capture), true) {
        Ok(seq) => {
            match seq.client_id() {
                Ok(our_id) => {
                    in_seq_available = true;

                    let ci = alsa::seq::ClientIter::new(&seq);
                    for client in ci {
                        if client.get_client() == our_id {
                            continue;
                        } // Skip ourselves

                        for port in alsa::seq::PortIter::new(&seq, client.get_client()) {
                            let caps = port.get_capability();

                            // Check that it's a normal input port
                            if !caps.contains(alsa::seq::PortCap::READ)
                                || !caps.contains(alsa::seq::PortCap::SUBS_READ)
                            {
                                continue;
                            }
                            if !port.get_type().contains(alsa::seq::PortType::MIDI_GENERIC) {
                                continue;
                            }

                            if let Ok(name) = port.get_name() {
                                server.in_devices.push(SystemMidiDeviceInfo {
                                    name: String::from(name),
                                });

                                info!("Found ALSA midi in port: {}", &name);
                            }
                        }
                    }
                }
                Err(e) => debug!("Could not get ID of current ALSA seq client: {}", e),
            }
        }
        Err(e) => info!("ALSA in seq server is unavailable: {}", e),
    }

    let mut out_seq_available = false;
    match alsa::Seq::open(None, Some(alsa::Direction::Playback), true) {
        Ok(seq) => {
            match seq.client_id() {
                Ok(our_id) => {
                    out_seq_available = true;

                    let ci = alsa::seq::ClientIter::new(&seq);
                    for client in ci {
                        if client.get_client() == our_id {
                            continue;
                        } // Skip ourselves

                        for port in alsa::seq::PortIter::new(&seq, client.get_client()) {
                            let caps = port.get_capability();

                            // Check that it's a normal input port
                            if !caps.contains(alsa::seq::PortCap::READ)
                                || !caps.contains(alsa::seq::PortCap::SUBS_READ)
                            {
                                continue;
                            }
                            if !port.get_type().contains(alsa::seq::PortType::MIDI_GENERIC) {
                                continue;
                            }

                            if let Ok(name) = port.get_name() {
                                server.out_devices.push(SystemMidiDeviceInfo {
                                    name: String::from(name),
                                });

                                info!("Found ALSA midi out port: {}", &name);
                            }
                        }
                    }
                }
                Err(e) => debug!("Could not get ID of current ALSA seq client: {}", e),
            }
        }
        Err(e) => info!("ALSA out seq server is unavailable: {}", e),
    }

    if in_seq_available || out_seq_available {
        server.available = true;
    } else {
        server.available = false;

        info!("ALSA midi server is unavailable");
    }
}

pub struct ALSARtThreadHandle<P: RtProcessHandler, E>
where
    E: 'static + Send + Sync + FnOnce(StreamError),
{
    _p: std::marker::PhantomData<P>,
    _e: std::marker::PhantomData<E>,
}

pub fn spawn_rt_thread<P: RtProcessHandler, E>(
    audio_config: &AudioServerConfig,
    create_midi_in_devices: &[MidiDeviceConfig],
    create_midi_out_devices: &[MidiDeviceConfig],
    mut rt_process_handler: P,
    error_callback: E,
    use_client_name: Option<String>,
) -> Result<(StreamInfo, ALSARtThreadHandle<P, E>), SpawnRtThreadError>
where
    E: 'static + Send + Sync + FnOnce(StreamError),
{
    info!("Spawning ALSA thread...");

    let client_name = use_client_name.unwrap_or(String::from("rusty-daw-io"));

    let mut in_pcm = None;
    let mut out_pcm = None;

    let mut audio_in_format = None;
    let mut audio_out_format = None;

    let mut audio_in_access = None;
    let mut audio_out_access = None;

    let mut sample_rate = 0;
    let mut buffer_size = 0;

    if audio_config.create_in_devices.len() > 0 {
        let in_device_name = audio_config.system_in_device.get_name_or({
            let mut first_available_device = None;

            for device_hint in HintIter::new(None, &*CString::new("pcm").unwrap())? {
                let device_name = match &device_hint.name {
                    None => continue,
                    Some(n) => {
                        if n == "null" {
                            continue;
                        }
                        n
                    }
                };

                let device_pretty_name = match &device_hint.desc {
                    None => continue,
                    Some(n) => extract_device_pretty_name(n),
                };

                if &device_pretty_name == &audio_config.system_duplex_device {
                    // Try to open device as input
                    if let Ok(pcm) =
                        alsa::pcm::PCM::new(device_name, alsa::Direction::Capture, true)
                    {
                        if let Ok(hwp) = alsa::pcm::HwParams::any(&pcm) {
                            if let Ok(_) = hwp.get_channels() {
                                first_available_device = Some(device_name.clone());
                                break;
                            }
                        }
                    }
                }
            }

            &first_available_device.ok_or_else(|| {
                SpawnRtThreadError::SystemAudioInDeviceNotFound(String::from("(auto)"))
            })?
        });

        let p = alsa::pcm::PCM::new(&in_device_name, alsa::Direction::Capture, true)?;

        // Set hardware parameters
        {
            let hwp = alsa::pcm::HwParams::any(&p)?;

            sample_rate = audio_config.sample_rate.unwrap_or({
                if hwp.test_rate(44_100).is_ok() {
                    44_100
                } else if hwp.test_rate(48_000).is_ok() {
                    48_000
                } else {
                    return Err(SpawnRtThreadError::CouldNotSetAutoSampleRate);
                }
            });
            hwp.set_rate(sample_rate, alsa::ValueOr::Nearest)?;
            if hwp.get_rate()? != sample_rate {
                return Err(SpawnRtThreadError::CouldNotSetSampleRate(sample_rate));
            }

            audio_in_format = Some(if let Ok(_) = hwp.set_format(alsa::pcm::Format::float()) {
                alsa::pcm::Format::float()
            } else if let Ok(_) = hwp.set_format(alsa::pcm::Format::s32()) {
                alsa::pcm::Format::s32()
            } else if let Ok(_) = hwp.set_format(alsa::pcm::Format::u32()) {
                alsa::pcm::Format::u32()
            } else if let Ok(_) = hwp.set_format(alsa::pcm::Format::s24()) {
                alsa::pcm::Format::s24()
            } else if let Ok(_) = hwp.set_format(alsa::pcm::Format::u24()) {
                alsa::pcm::Format::u24()
            } else if let Ok(_) = hwp.set_format(alsa::pcm::Format::s16()) {
                alsa::pcm::Format::s16()
            } else if let Ok(_) = hwp.set_format(alsa::pcm::Format::u16()) {
                alsa::pcm::Format::u16()
            } else if let Ok(_) = hwp.set_format(alsa::pcm::Format::U8) {
                alsa::pcm::Format::U8
            } else {
                return Err(SpawnRtThreadError::Other(format!(
                    "Could not find compatible sample format for ALSA device {}",
                    in_device_name
                )));
            });

            audio_in_access = Some(
                if let Ok(_) = hwp.set_access(alsa::pcm::Access::MMapNonInterleaved) {
                    alsa::pcm::Access::MMapNonInterleaved
                } else if let Ok(_) = hwp.set_access(alsa::pcm::Access::MMapInterleaved) {
                    alsa::pcm::Access::MMapNonInterleaved
                } else {
                    return Err(SpawnRtThreadError::Other(format!(
                        "Could not set access to ALSA device {}",
                        in_device_name
                    )));
                },
            );

            buffer_size = audio_config.max_buffer_size.unwrap_or(512);
            hwp.set_buffer_size(buffer_size as i64)?;
            hwp.set_period_size((buffer_size / 2) as i64, alsa::ValueOr::Nearest)?;

            p.hw_params(&hwp)?;
        }

        // Set software parameters
        {
            let hwp = p.hw_params_current()?;
            let swp = p.sw_params_current()?;
            let (bufsize, periodsize) = (hwp.get_buffer_size()?, hwp.get_period_size()?);

            swp.set_start_threshold(bufsize - periodsize)?;
            swp.set_avail_min(periodsize)?;

            p.sw_params(&swp)?;
        }

        in_pcm = Some(p);
    }

    if audio_config.create_out_devices.len() > 0 {
        let out_device_name = audio_config.system_out_device.get_name_or({
            let mut first_available_device = None;

            for device_hint in HintIter::new(None, &*CString::new("pcm").unwrap())? {
                let device_name = match &device_hint.name {
                    None => continue,
                    Some(n) => {
                        if n == "null" {
                            continue;
                        }
                        n
                    }
                };

                let device_pretty_name = match &device_hint.desc {
                    None => continue,
                    Some(n) => extract_device_pretty_name(n),
                };

                if &device_pretty_name == &audio_config.system_duplex_device {
                    // Try to open device as output
                    if let Ok(pcm) =
                        alsa::pcm::PCM::new(device_name, alsa::Direction::Playback, true)
                    {
                        if let Ok(hwp) = alsa::pcm::HwParams::any(&pcm) {
                            if let Ok(_) = hwp.get_channels() {
                                first_available_device = Some(device_name.clone());
                                break;
                            }
                        }
                    }
                }
            }

            &first_available_device.ok_or_else(|| {
                SpawnRtThreadError::SystemAudioOutDeviceNotFound(String::from("(auto)"))
            })?
        });

        let p = alsa::pcm::PCM::new(&out_device_name, alsa::Direction::Playback, true)?;

        // Set hardware parameters
        {
            let hwp = alsa::pcm::HwParams::any(&p)?;

            sample_rate = audio_config.sample_rate.unwrap_or({
                if hwp.test_rate(44_100).is_ok() {
                    44_100
                } else if hwp.test_rate(48_000).is_ok() {
                    48_000
                } else {
                    return Err(SpawnRtThreadError::CouldNotSetAutoSampleRate);
                }
            });
            hwp.set_rate(sample_rate, alsa::ValueOr::Nearest)?;
            if hwp.get_rate()? != sample_rate {
                return Err(SpawnRtThreadError::CouldNotSetSampleRate(sample_rate));
            }

            audio_out_format = Some(if let Ok(_) = hwp.set_format(alsa::pcm::Format::float()) {
                alsa::pcm::Format::float()
            } else if let Ok(_) = hwp.set_format(alsa::pcm::Format::s32()) {
                alsa::pcm::Format::s32()
            } else if let Ok(_) = hwp.set_format(alsa::pcm::Format::u32()) {
                alsa::pcm::Format::u32()
            } else if let Ok(_) = hwp.set_format(alsa::pcm::Format::s24()) {
                alsa::pcm::Format::s24()
            } else if let Ok(_) = hwp.set_format(alsa::pcm::Format::u24()) {
                alsa::pcm::Format::u24()
            } else if let Ok(_) = hwp.set_format(alsa::pcm::Format::s16()) {
                alsa::pcm::Format::s16()
            } else if let Ok(_) = hwp.set_format(alsa::pcm::Format::u16()) {
                alsa::pcm::Format::u16()
            } else if let Ok(_) = hwp.set_format(alsa::pcm::Format::U8) {
                alsa::pcm::Format::U8
            } else {
                return Err(SpawnRtThreadError::Other(format!(
                    "Could not find compatible sample format for ALSA device {}",
                    out_device_name
                )));
            });

            audio_out_access = Some(
                if let Ok(_) = hwp.set_access(alsa::pcm::Access::MMapNonInterleaved) {
                    alsa::pcm::Access::MMapNonInterleaved
                } else if let Ok(_) = hwp.set_access(alsa::pcm::Access::MMapInterleaved) {
                    alsa::pcm::Access::MMapNonInterleaved
                } else {
                    return Err(SpawnRtThreadError::Other(format!(
                        "Could not set access to ALSA device {}",
                        out_device_name
                    )));
                },
            );

            buffer_size = audio_config.max_buffer_size.unwrap_or(512);
            hwp.set_buffer_size(buffer_size as i64)?;
            hwp.set_period_size((buffer_size / 2) as i64, alsa::ValueOr::Nearest)?;

            p.hw_params(&hwp)?;
        }

        // Set software parameters
        {
            let hwp = p.hw_params_current()?;
            let swp = p.sw_params_current()?;
            let (bufsize, periodsize) = (hwp.get_buffer_size()?, hwp.get_period_size()?);

            swp.set_start_threshold(bufsize - periodsize)?;
            swp.set_avail_min(periodsize)?;

            p.sw_params(&swp)?;
        }

        out_pcm = Some(p);
    }

    let stream_info = StreamInfo {
        server_name: String::from("ALSA"),
        audio_in: vec![],
        audio_out: vec![],
        midi_in: vec![],
        midi_out: vec![],
        sample_rate: sample_rate as u32,
        audio_buffer_size: BufferSizeInfo::UnknownSize,
    };

    Ok((
        stream_info,
        ALSARtThreadHandle {
            _p: std::marker::PhantomData,
            _e: std::marker::PhantomData,
        },
    ))
}

impl From<alsa::Error> for SpawnRtThreadError {
    fn from(e: alsa::Error) -> Self {
        SpawnRtThreadError::PlatformSpecific(Box::new(e))
    }
}
