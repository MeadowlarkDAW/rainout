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
    info!("Refreshing list of available ALSA audio devices...");

    server.system_in_devices.clear();
    server.system_out_devices.clear();

    use alsa::device_name::HintIter;
    use std::ffi::CString;

    match HintIter::new(None, &*CString::new("pcm").unwrap()) {
        Ok(i) => {
            for device_hint in i {
                let name = match &device_hint.name {
                    None => continue,
                    Some(n) => {
                        if n == "null" {
                            continue;
                        }
                        n
                    }
                };

                /*
                // Try to open device as input
                if let Ok(_) = alsa::pcm::PCM::new(name, alsa::Direction::Capture, true) {
                    server.system_in_devices.push(SystemAudioDeviceInfo {
                        name: name.clone(),
                        ports: vec![String::from("left"), String::from("right")],  // TODO: Get actual channels?
                    })
                }

                // Try to open device as output
                if let Ok(_) = alsa::pcm::PCM::new(name, alsa::Direction::Playback, true) {
                    server.system_out_devices.push(SystemAudioDeviceInfo {
                        name: name.clone(),
                        ports: vec![String::from("left"), String::from("right")],  // TODO: Get actual channels?
                    })
                }
                */
            }
        }
        Err(_) => {
            info!("ALSA server is unavailable");
        }
    }
}
