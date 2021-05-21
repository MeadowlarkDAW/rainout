use rusty_daw_io::{
    AudioDeviceConfig, AudioServerConfig, DeviceInfo, ProcessInfo, RtProcessHandler, StreamInfo,
};

fn main() {
    let info = DeviceInfo::new();

    dbg!(info.audio_server_info());
    dbg!(info.midi_server_info());

    let audio_config = AudioServerConfig {
        server_name: String::from("Jack"),
        use_in_devices: vec![
            AudioDeviceConfig {
                device_name: String::from("system:capture_1"),
                use_num_channels: None,
            },
            AudioDeviceConfig {
                device_name: String::from("system:capture_2"),
                use_num_channels: None,
            },
        ],
        use_out_devices: vec![
            AudioDeviceConfig {
                device_name: String::from("system:playback_1"),
                use_num_channels: None,
            },
            AudioDeviceConfig {
                device_name: String::from("system:playback_2"),
                use_num_channels: None,
            },
        ],
        use_sample_rate: None,
        use_buffer_size: None,
    };

    let stream_handle =
        rusty_daw_io::spawn_rt_thread(&audio_config, None, None, MyRtProcessHandler {}, |e| {
            println!("Fatal stream error: {:?}", e);
        })
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
