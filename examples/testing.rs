use rusty_daw_io::{
    AudioDeviceConfig, AudioServerConfig, DevicesInfo, MidiDeviceConfig, MidiServerConfig,
    ProcessInfo, RtProcessHandler, StreamInfo, UseName, UseU32,
};

fn main() {
    simple_logger::SimpleLogger::new().init().unwrap();

    let info = DevicesInfo::new();

    dbg!(info.audio_servers_info());
    dbg!(info.midi_servers_info());

    let audio_config = AudioServerConfig {
        server: UseName::Auto,
        system_duplex_device: UseName::Auto,
        system_in_device: UseName::Auto,
        system_out_device: UseName::Auto,

        create_in_devices: vec![AudioDeviceConfig {
            id: String::from("audio_in"),
            system_channels: vec![0, 1],
        }],
        create_out_devices: vec![AudioDeviceConfig {
            id: String::from("audio_out"),
            system_channels: vec![0, 1],
        }],

        sample_rate: UseU32::Auto,
        max_buffer_size: UseU32::Auto,
    };

    let midi_config = MidiServerConfig {
        server: UseName::Auto,
        create_in_devices: vec![MidiDeviceConfig {
            id: String::from("midi_in"),
            system_port: UseName::Use(String::from("system:midi_capture_2")),
        }],
        create_out_devices: vec![MidiDeviceConfig {
            id: String::from("midi_out"),
            system_port: UseName::Auto,
        }],
    };

    let stream_handle = rusty_daw_io::spawn_rt_thread(
        &audio_config,
        Some(&midi_config),
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
    fn init(&mut self, stream_info: &StreamInfo) {}
    fn process(&mut self, proc_info: ProcessInfo) {}
}
