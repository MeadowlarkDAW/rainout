use log::{debug, info, warn};

use crate::{
    AudioDeviceBuffer, AudioDeviceConfig, AudioServerInfo, BufferSizeInfo, DeviceIndex,
    InternalAudioDevice, InternalMidiDevice, MidiDeviceBuffer, MidiDeviceConfig, MidiServerInfo,
    ProcessInfo, RtProcessHandler, SpawnRtThreadError, StreamError, StreamInfo,
    SystemAudioDeviceInfo, SystemMidiDeviceInfo,
};

fn extract_device_name(desc: &String) -> String {
    let mut i = 0;
    for c in desc.chars() {
        i += 1;
        if c == '\n' {
            break;
        }
    }

    String::from(&desc[0..i])
}

pub fn refresh_audio_server(server: &mut AudioServerInfo) {
    /*
    info!("Refreshing list of available ALSA audio devices...");

    server.system_in_devices.clear();
    server.system_out_devices.clear();

    use alsa::device_name::HintIter;
    use std::ffi::CString;

    match HintIter::new(None, &*CString::new("pcm").unwrap()) {
        Ok(i) => {
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

                let mut min_buffer_size = u32::MAX;
                let mut max_buffer_size = 0;

                // Try to open device as input
                if let Ok(pcm) = alsa::pcm::PCM::new(device_name, alsa::Direction::Capture, true) {
                    match alsa::pcm::HwParams::any(&pcm) {
                        Ok(hw_params) => {
                            match hw_params.get_channels() {
                                Ok(ch) => {
                                    match hw_params.get_buffer_size_min() {
                                        Ok(b) => {
                                            min_buffer_size = min_buffer_size.min(b as u32);
                                        }
                                        Err(e) => debug!("Could not get min buffer size of ALSA device {}: {}", device_name, e)
                                    }

                                    server.system_in_devices.push(SystemAudioDeviceInfo {
                                        name: device_name.clone(),
                                        channels: ch as u16,
                                    });
                                }
                                Err(e) => debug!("Could not get channels of ALSA device {}: {}", device_name, e),
                            }
                        }
                        Err(e) => debug!("Could not get params of ALSA device {}: {}", device_name, e),
                    }
                }

                // Try to open device as output
                if let Ok(_) = alsa::pcm::PCM::new(device_name, alsa::Direction::Playback, true) {

                }
            }
        }
        Err(_) => {
            info!("ALSA server is unavailable");
        }
    }
    */
}
