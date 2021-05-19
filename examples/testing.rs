use rusty_daw_io::{DeviceConfigurator, ProcessInfo, RtProcessHandler};

fn main() {
    let mut config = DeviceConfigurator::new(None);

    config
        .server_configs_mut()
        .first_mut()
        .unwrap()
        .set_selected(true);
    let jack_server_config = config.server_configs_mut().first_mut().unwrap();

    jack_server_config.set_selected(true);

    if let Some(jack_device_config) = jack_server_config.audio_devices_mut().first_mut() {
        jack_device_config.set_selected(true);
        jack_device_config.set_output_channels(Some(4));
    }

    dbg!(config.server_configs());

    let res = config.spawn_rt_thread(MyRtProcessHandler {}, |e| {
        println!("Fatal stream error: {:?}", e);
    });

    match &res {
        Ok(stream_handle) => {}
        Err((config, e)) => {
            println!("Error opening stream: {:?}", e);
        }
    }

    // Wait for user input to quit
    println!("Press enter/return to quit...");
    let mut user_input = String::new();
    std::io::stdin().read_line(&mut user_input).ok();
}

struct MyRtProcessHandler {}

impl RtProcessHandler for MyRtProcessHandler {
    fn process(&mut self, proc_info: ProcessInfo) {}
}
