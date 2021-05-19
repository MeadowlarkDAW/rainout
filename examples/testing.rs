use rusty_daw_io::{DeviceConfigurator, ProcessInfo};

fn main() {
    let mut config = DeviceConfigurator::<_>::new(None);

    config.server_configs_mut().first_mut().unwrap().set_selected(true);
    let jack_server_config = config.server_configs_mut().first_mut().unwrap();

    jack_server_config.set_selected(true);
    
    if let Some(jack_device_config) = jack_server_config.audio_devices_mut().first_mut() {
        jack_device_config.set_selected(true);
    }

    config.spawn_rt_thread(|proc_info: ProcessInfo| {}).unwrap();

    // Wait for user input to quit
    println!("Press enter/return to quit...");
    let mut user_input = String::new();
    std::io::stdin().read_line(&mut user_input).ok();
}
