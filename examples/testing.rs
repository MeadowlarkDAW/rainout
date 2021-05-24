use rusty_daw_io::{
    AudioDeviceConfig, AudioServerConfig, ConnectionType, DeviceInfo, MidiDeviceConfig,
    MidiServerConfig, ProcessInfo, RtProcessHandler, StreamInfo,
};

fn main() {
    let info = DeviceInfo::new();

    dbg!(info.audio_server_info());
    //dbg!(info.midi_server_info());

    let audio_config = AudioServerConfig {
        server_name: String::from("Jack"),
        use_in_devices: vec![AudioDeviceConfig {
            id: String::from("default_in"),
            connection: ConnectionType::SystemPorts {
                ports: vec![
                    String::from("system:capture_1"),
                    String::from("system:capture_2"),
                ],
            },
        }],
        use_out_devices: vec![AudioDeviceConfig {
            id: String::from("default_out"),
            connection: ConnectionType::SystemPorts {
                ports: vec![
                    String::from("system:playback_1"),
                    String::from("system:playback_2"),
                ],
            },
        }],
        use_sample_rate: None,
        use_max_buffer_size: None,
    };

    /*
    let midi_config = MidiServerConfig {
        server_name: String::from("Jack"),
        use_in_devices: vec![MidiDeviceConfig {
            device_name: String::from("system:midi_capture_2"),
        }],
        use_out_devices: vec![MidiDeviceConfig {
            device_name: String::from("system:midi_playback_2"),
        }],
        buffer_size_bytes: 2048,
    };
    */

    let stream_handle = rusty_daw_io::spawn_rt_thread(
        &audio_config,
        None,
        Some(String::from("testing")),
        MyRtProcessHandler {},
        |e| {
            println!("Fatal stream error: {:?}", e);
        },
    )
    .unwrap();

    dbg!(stream_handle.stream_info());

    // Wait for user input to quit
    println!("Press enter/return to quit...");
    let mut user_input = String::new();
    std::io::stdin().read_line(&mut user_input).ok();
}

struct MyRtProcessHandler {}

impl RtProcessHandler for MyRtProcessHandler {
    fn init(&mut self, stream_info: &StreamInfo) {
        println!("init");
        dbg!(stream_info);
    }
    fn process(&mut self, proc_info: ProcessInfo) {}
}
