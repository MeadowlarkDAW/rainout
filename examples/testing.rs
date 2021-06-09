use rusty_daw_io::{
    AudioBusConfig, Config, DevicesInfo, FatalErrorHandler, FatalStreamError, MidiControllerConfig,
    ProcessInfo, RtProcessHandler, StreamInfo,
};

fn main() {
    simple_logger::SimpleLogger::new().init().unwrap();

    let info = DevicesInfo::new();

    dbg!(info.audio_servers_info());
    dbg!(info.midi_servers_info());

    let config = Config {
        audio_server: String::from("Jack"),
        system_audio_device: String::from("Jack"),

        audio_in_busses: vec![AudioBusConfig {
            id: String::from("audio_in"),
            system_ports: vec![
                String::from("system:capture_1"),
                String::from("system:capture_2"),
            ],
        }],
        audio_out_busses: vec![AudioBusConfig {
            id: String::from("audio_out"),
            system_ports: vec![
                String::from("system:playback_1"),
                String::from("system:playback_2"),
            ],
        }],

        sample_rate: None,
        buffer_size: None,

        midi_server: Some(String::from("Jack")),

        midi_in_controllers: vec![MidiControllerConfig {
            id: String::from("midi_in"),
            system_port: String::from("system:midi_capture_2"),
        }],

        midi_out_controllers: vec![MidiControllerConfig {
            id: String::from("midi_out"),
            system_port: String::from("system:midi_playback_1"),
        }],
    };

    let stream_handle = rusty_daw_io::spawn_rt_thread(
        &config,
        Some(String::from("testing")),
        MyRtProcessHandler {},
        MyFatalErrorHandler {},
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

struct MyFatalErrorHandler {}

impl FatalErrorHandler for MyFatalErrorHandler {
    fn fatal_stream_error(self, error: FatalStreamError) {
        println!("Fatal stream error: {}", error);
    }
}
